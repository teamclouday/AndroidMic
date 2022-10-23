package com.example.androidmic

import android.app.Application
import android.content.*
import android.os.IBinder
import android.os.Messenger
import android.util.Log
import com.example.androidmic.domain.service.ForegroundService
import com.example.androidmic.ui.MainActivity


class AndroidMicApp : Application() {
    private val TAG = "AndroidMicApp"

    var mService: Messenger? = null
    var mBound = false

    val mConnection = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName?, service: IBinder?) {
            Log.d(TAG, "onServiceConnected")
            mService = Messenger(service)
            mBound = true
            // notify current running activity that service is connected
            val notifyIntent = Intent(applicationContext, MainActivity::class.java).apply {
                action = Intent.ACTION_VIEW
                addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_NEW_TASK)
                putExtra("ForegroundServiceBound", true)
            }
            startActivity(notifyIntent)
        }

        override fun onServiceDisconnected(name: ComponentName?) {
            Log.d(TAG, "onServiceDisconnected")
            mService = null
            mBound = false
        }
    }

    override fun onCreate() {
        super.onCreate()
        Log.d(TAG, "onCreate")
        bindService()
    }

    // start and bind to service
    fun bindService() {
        val intent = Intent(this, ForegroundService::class.java)
        startService(intent)
        bindService(intent, mConnection, Context.BIND_AUTO_CREATE)
    }
}