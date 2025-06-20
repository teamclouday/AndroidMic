package io.github.teamclouday.androidMic

import android.app.Application
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.IBinder
import android.os.Messenger
import android.util.Log
import io.github.teamclouday.androidMic.domain.service.ForegroundService
import io.github.teamclouday.androidMic.ui.MainActivity
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.launch


class AndroidMicApp : Application() {
    private val TAG = "AndroidMicApp"


    companion object {
        lateinit var appModule: AppModule
        var mService: Messenger? = null
        var mBound = false
    }

    private val scope = MainScope()
    override fun onCreate() {
        super.onCreate()

        appModule = AppModuleImpl(this)

        scope.launch {
            appModule.appPreferences.preload()
        }
    }

    private val mConnection = object : ServiceConnection {
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

    // start and bind to service
    fun bindService() {
        val intent = Intent(this, ForegroundService::class.java)
        startService(intent)
        bindService(intent, mConnection, Context.BIND_AUTO_CREATE)
    }

    fun unBindService() {
        unbindService(mConnection)
        mService = null
        mBound = false
    }
}