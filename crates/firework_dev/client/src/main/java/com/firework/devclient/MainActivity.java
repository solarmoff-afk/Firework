// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

package com.firework.devclient;

import android.app.Activity;
import android.os.Bundle;

public class MainActivity extends Activity {
    private RenderView renderView;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        renderView = new RenderView(this);
        setContentView(renderView);
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        // renderView.stopServer();
    }
}