package io.github.teamclouday.AndroidMic

import android.app.Application
import android.content.ComponentName
import android.content.Intent
import android.content.ServiceConnection
import android.os.IBinder
import android.os.Messenger
import android.util.Log
import io.github.teamclouday.AndroidMic.domain.service.BIND_SERVICE_ACTION
import io.github.teamclouday.AndroidMic.domain.service.ForegroundService
import io.github.teamclouday.AndroidMic.ui.MainActivity
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.launch


class AndroidMicApp : Application() {
    private val TAG = "AndroidMicApp"


    companion object {
        lateinit var appModule: AppModule
        var service: Messenger? = null
        var isBound = false
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
            Companion.service = Messenger(service)
            isBound = true
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
            service = null
            isBound = false
        }
    }

    // start and bind to service
    fun bindService() {
        val intent = Intent(this, ForegroundService::class.java).apply {
            action = BIND_SERVICE_ACTION
        }
        startService(intent)
        bindService(intent, mConnection, BIND_AUTO_CREATE)
    }

    fun unBindService() {
        unbindService(mConnection)
        service = null
        isBound = false
    }
}