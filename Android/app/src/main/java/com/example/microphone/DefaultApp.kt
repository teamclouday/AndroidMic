package com.example.microphone

import android.app.Application
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.IBinder
import android.os.Messenger

class DefaultApp : Application()
{
    var mService : Messenger? = null
    var mBound = false

    private val mConnection = object : ServiceConnection
    {
        override fun onServiceConnected(name: ComponentName?, service: IBinder?)
        {
            mService = Messenger(service)
            mBound = true
            // notify current running activity that service is connected
            val notifyIntent = Intent(applicationContext, MainActivity::class.java).apply {
                action = Intent.ACTION_VIEW
                addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_NEW_TASK)
                putExtra("BackgroundHelperBound", true)
            }
            startActivity(notifyIntent)
        }

        override fun onServiceDisconnected(name: ComponentName?)
        {
            mService = null
            mBound = false
        }
    }

    override fun onCreate() {
        super.onCreate()
        // start and bind to service
        val intent = Intent(this, BackgroundHelper::class.java)
        // startService(intent)
        bindService(intent, mConnection, Context.BIND_AUTO_CREATE)
    }
}