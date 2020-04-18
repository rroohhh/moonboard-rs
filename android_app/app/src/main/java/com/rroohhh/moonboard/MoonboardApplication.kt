package com.rroohhh.moonboard

import android.app.Application

class MoonboardApplication : Application() {
    companion object {
        lateinit var board: MoonboardJavaGlue;
    }

    override fun onCreate() {
        super.onCreate()

        System.loadLibrary("moonboard")
    }
}