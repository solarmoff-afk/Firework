// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

package com.firework.devclient;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.graphics.Path;
import android.graphics.RectF;
import android.graphics.Typeface;
import android.os.Handler;
import android.os.Looper;
import android.text.Layout;
import android.text.StaticLayout;
import android.text.TextPaint;
import android.view.Choreographer;
import android.view.MotionEvent;
import android.view.View;
import android.widget.Toast;

import org.json.JSONArray;
import org.json.JSONObject;

import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.net.ServerSocket;
import java.net.Socket;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public class RenderView extends View implements Choreographer.FrameCallback {
    private ServerSocket serverSocket;
    private Socket clientSocket;
    private OutputStream outStream;
    private Thread serverThread;
    
    private final Handler mainHandler = new Handler(Looper.getMainLooper());
    private final ExecutorService networkExecutor = Executors.newSingleThreadExecutor();
    
    private boolean isRunLoopActive = false;

    private final ConcurrentHashMap<Integer, FWPrimitive> primitives = new ConcurrentHashMap<>();

    private static class FWPrimitive {
        int id;
        boolean isText;
        RectF rect = new RectF(0, 0, 0, 0);
        Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);
        TextPaint textPaint = new TextPaint(Paint.ANTI_ALIAS_FLAG);
        
        int zIndex = 0;
        boolean visible = true;
        int hitGroup = 0;
        int clipToId = -1; 
        
        float[] radii = new float[8];
        float borderWidth = 0;
        int borderColor = Color.TRANSPARENT;
        
        StringBuilder text = new StringBuilder();
        int textAlign = 0; 
        int wrapWidth = 0;
        
        FWPrimitive(int id, boolean isText) {
            this.id = id;
            this.isText = isText;
            paint.setStyle(Paint.Style.FILL);
            textPaint.setTextSize(14);
        }
    }

    public RenderView(Context context) {
        super(context);
        setLayerType(LAYER_TYPE_SOFTWARE, null);
        startServer();
    }

    private void startServer() {
        serverThread = new Thread(() -> {
            try {
                serverSocket = new ServerSocket(9090);
                serverSocket.setReuseAddress(true);
                showToast("Server started on 9090");

                while (!Thread.currentThread().isInterrupted()) {
                    clientSocket = serverSocket.accept();
                    outStream = clientSocket.getOutputStream();
                    showToast("Connected");

                    BufferedReader reader = new BufferedReader(new InputStreamReader(clientSocket.getInputStream()));
                    String line;
                    while ((line = reader.readLine()) != null) {
                        processCommand(line);
                        post(this::invalidate);
                    }
                }
            } catch (Exception e) {
                showToast("Error: " + e.getMessage());
            }
        });
        serverThread.start();
    }

    private void processCommand(String jsonStr) {
        try {
            JSONObject obj = new JSONObject(jsonStr);
            String cmd = obj.getString("cmd");

            if (cmd.equals("RemoveAll")) {
                primitives.clear();
                return;
            }
        	
            if (cmd.equals("RunLoop")) {
                if (!isRunLoopActive) {
                    isRunLoopActive = true;
                    Choreographer.getInstance().postFrameCallback(this);
                }
                return;
            }

            int id = obj.optInt("id", -1);
            if (id == -1 && !cmd.equals("ResolveHit")) return;

            switch (cmd) {
                case "NewRect":
                    primitives.put(id, new FWPrimitive(id, false));
                    break;
                case "NewText":
                    primitives.put(id, new FWPrimitive(id, true));
                    break;
                case "SetPosition":
                    JSONArray pos = obj.getJSONArray("pos");
                    if (primitives.containsKey(id)) {
                        FWPrimitive p = primitives.get(id);
                        float w = p.rect.width(), h = p.rect.height();
                        p.rect.set((float)pos.getDouble(0), (float)pos.getDouble(1), (float)pos.getDouble(0)+w, (float)pos.getDouble(1)+h);
                    }
                    break;
                case "SetSize":
                    JSONArray size = obj.getJSONArray("size");
                    if (primitives.containsKey(id)) {
                        FWPrimitive p = primitives.get(id);
                        p.rect.right = p.rect.left + (float)size.getDouble(0);
                        p.rect.bottom = p.rect.top + (float)size.getDouble(1);
                    }
                	
                    break;
                case "SetColor":
                    JSONArray c = obj.getJSONArray("color");
                    if (primitives.containsKey(id)) {
                        int color = Color.argb(c.getInt(3), c.getInt(0), c.getInt(1), c.getInt(2));
                        primitives.get(id).paint.setColor(color);
                        primitives.get(id).textPaint.setColor(color);
                    }
                    break;
                case "SetZ":
                    if (primitives.containsKey(id)) primitives.get(id).zIndex = obj.getInt("z");
                    break;
                case "SetVisible":
                    if (primitives.containsKey(id)) primitives.get(id).visible = obj.getBoolean("vis");
                    break;
                case "SetClipTo":
                    if (primitives.containsKey(id)) primitives.get(id).clipToId = obj.getInt("clip_id");
                    break;
                case "Remove":
                    primitives.remove(id);
                    break;
                case "SetHitGroup":
                    if (primitives.containsKey(id)) primitives.get(id).hitGroup = obj.getInt("group");
                    break;
                case "ResolveHit":
                    int reqGroup = obj.getInt("group");
                    JSONArray r = obj.getJSONArray("rect");
                    RectF reqRect = new RectF((float)r.getDouble(0), (float)r.getDouble(1), 
                            (float)r.getDouble(0)+(float)r.getDouble(2), (float)r.getDouble(1)+(float)r.getDouble(3));
                    
                    List<FWPrimitive> list = new ArrayList<>(primitives.values());
                    Collections.sort(list, (a, b) -> Integer.compare(b.zIndex, a.zIndex));
                    
                    int hitId = -1;
                    for (FWPrimitive p : list) {
                        if (p.visible && p.hitGroup == reqGroup && RectF.intersects(p.rect, reqRect)) {
                            hitId = p.id;
                            break;
                        }
                    }
                    sendToRust("{\"res\":\"Hit\",\"id\":" + hitId + "}");
                    break;
                case "PushText":
                    if (primitives.containsKey(id)) {
                        FWPrimitive p = primitives.get(id);
                        p.text.append(obj.getString("text"));
                        int mode = obj.getInt("mode");
                        if (mode == 1) p.textPaint.setTypeface(Typeface.create(Typeface.DEFAULT, Typeface.BOLD));
                        else if (mode == 2) p.textPaint.setTypeface(Typeface.create(Typeface.DEFAULT, Typeface.ITALIC));
                        else if (mode == 3) p.textPaint.setTypeface(Typeface.create(Typeface.DEFAULT, Typeface.BOLD_ITALIC));
                        else p.textPaint.setTypeface(Typeface.create(Typeface.DEFAULT, Typeface.NORMAL));
                    }
                    break;
                case "ClearText":
                    if (primitives.containsKey(id)) primitives.get(id).text.setLength(0);
                    break;
                case "SetTextAlign":
                    if (primitives.containsKey(id)) primitives.get(id).textAlign = obj.getInt("align");
                    break;
                case "SetTextWrapWidth":
                    if (primitives.containsKey(id)) primitives.get(id).wrapWidth = obj.getInt("width");
                    break;
                case "MeasureText":
                    if (primitives.containsKey(id)) {
                        FWPrimitive p = primitives.get(id);
                        String txt = p.text.toString();
                        if (p.wrapWidth > 0) {
                            StaticLayout sl = new StaticLayout(txt, p.textPaint, p.wrapWidth, Layout.Alignment.ALIGN_NORMAL, 1.0f, 0.0f, false);
                            sendToRust("{\"res\":\"Size\",\"w\":" + sl.getWidth() + ",\"h\":" + sl.getHeight() + "}");
                        } else {
                            float w = p.textPaint.measureText(txt);
                            Paint.FontMetrics fm = p.textPaint.getFontMetrics();
                            float h = fm.descent - fm.ascent;
                            sendToRust("{\"res\":\"Size\",\"w\":" + Math.round(w) + ",\"h\":" + Math.round(h) + "}");
                        }
                    } else {
                        sendToRust("{\"res\":\"Size\",\"w\":0,\"h\":0}");
                    }
                    break;
                case "SetCornerRadius":
                    if (primitives.containsKey(id)) {
                        JSONArray rad = obj.optJSONArray("radius");
                        if (rad == null) rad = obj.getJSONArray("rad"); 
                        float tl = (float)rad.getDouble(0), tr = (float)rad.getDouble(1),
                              br = (float)rad.getDouble(2), bl = (float)rad.getDouble(3);
                        primitives.get(id).radii = new float[]{tl, tl, tr, tr, br, br, bl, bl};
                    }
                    break;
                case "SetBorder":
                    if (primitives.containsKey(id)) {
                        primitives.get(id).borderWidth = (float)obj.getDouble("width");
                        JSONArray bc = obj.getJSONArray("color");
                        primitives.get(id).borderColor = Color.argb(bc.getInt(3), bc.getInt(0), bc.getInt(1), bc.getInt(2));
                    }
                    break;
                case "SetFontSize":
                    if (primitives.containsKey(id)) {
                        primitives.get(id).textPaint.setTextSize((float)obj.getDouble("size"));
                    }
                    break;
                case "SetShadow":
                    if (primitives.containsKey(id)) {
                        JSONArray off = obj.optJSONArray("offset");
                        if (off == null) off = obj.getJSONArray("off");
                        JSONArray sc = obj.getJSONArray("color");
                        primitives.get(id).paint.setShadowLayer(
                                (float)obj.getDouble("blur"), (float)off.getDouble(0), (float)off.getDouble(1),
                                Color.argb(sc.getInt(3), sc.getInt(0), sc.getInt(1), sc.getInt(2))
                        );
                    }
                    break;
            }
        } catch (Exception ignored) { }
    }

    private void sendToRust(String msg) {
        networkExecutor.execute(() -> {
            if (outStream != null) {
                try {
                    outStream.write((msg + "\n").getBytes());
                    outStream.flush();
                } catch (Exception ignored) {}
            }
        });
    }

    @Override
    public boolean onTouchEvent(MotionEvent event) {
        int phase = 3; 
        switch (event.getAction()) {
            case MotionEvent.ACTION_DOWN: phase = 0; break;
            case MotionEvent.ACTION_MOVE: phase = 1; break;
            case MotionEvent.ACTION_UP: phase = 2; break;
        }
        sendToRust(String.format("{\"evt\":\"Touch\",\"x\":%d,\"y\":%d,\"phase\":%d}", 
                (int)event.getX(), (int)event.getY(), phase));
        return true;
    }

    @Override
    public void doFrame(long frameTimeNanos) {
        if (isRunLoopActive) {
            sendToRust("{\"evt\":\"Tick\"}");
            Choreographer.getInstance().postFrameCallback(this);
        }
    }

    @Override
    protected void onDraw(Canvas canvas) {
        super.onDraw(canvas);
        canvas.drawColor(Color.WHITE);

        List<FWPrimitive> list = new ArrayList<>(primitives.values());
        Collections.sort(list, (a, b) -> Integer.compare(a.zIndex, b.zIndex));

        for (FWPrimitive p : list) {
            if (!p.visible) continue;

            canvas.save();
            if (p.clipToId != -1 && primitives.containsKey(p.clipToId)) {
                FWPrimitive parent = primitives.get(p.clipToId);
                Path clipPath = new Path();
                clipPath.addRoundRect(parent.rect, parent.radii, Path.Direction.CW);
                canvas.clipPath(clipPath);
            }

            if (!p.isText) {
                Path path = new Path();
                path.addRoundRect(p.rect, p.radii, Path.Direction.CW);
                canvas.drawPath(path, p.paint);

                if (p.borderWidth > 0) {
                    Paint stroke = new Paint(Paint.ANTI_ALIAS_FLAG);
                    stroke.setStyle(Paint.Style.STROKE);
                    stroke.setStrokeWidth(p.borderWidth);
                    stroke.setColor(p.borderColor);
                    canvas.drawPath(path, stroke);
                }
            } else {
                canvas.translate(p.rect.left, p.rect.top);
                Layout.Alignment align = p.textAlign == 1 ? Layout.Alignment.ALIGN_CENTER : 
                                        p.textAlign == 2 ? Layout.Alignment.ALIGN_OPPOSITE : Layout.Alignment.ALIGN_NORMAL;
                
                String txt = p.text.toString();
                if (p.wrapWidth > 0) {
                    StaticLayout sl = new StaticLayout(txt, p.textPaint, p.wrapWidth, align, 1.0f, 0.0f, false);
                    sl.draw(canvas);
                } else {
                    Paint.FontMetrics fm = p.textPaint.getFontMetrics();
                    float y = Math.abs(fm.ascent);
                    canvas.drawText(txt, 0, y, p.textPaint);
                }
            }
            canvas.restore();
        }
    }

    private void showToast(final String message) {
        mainHandler.post(() -> Toast.makeText(getContext(), message, Toast.LENGTH_SHORT).show());
    }

    public void stopServer() {
        isRunLoopActive = false;
        if (serverThread != null) serverThread.interrupt();
        try { if (serverSocket != null) serverSocket.close(); } catch (Exception ignored) {}
    }
}
