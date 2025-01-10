package com.example.androidMic.domain.service

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.HandlerThread
import android.os.IBinder
import android.os.Looper
import android.os.Message
import android.os.Messenger
import android.os.Process
import android.util.Log
import com.example.androidMic.Mode
import com.example.androidMic.R
import com.example.androidMic.domain.audio.MicAudioManager
import com.example.androidMic.domain.streaming.MicStreamManager
import com.example.androidMic.utils.ignore
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch


data class ServiceStates(
    var isStreamStarted: Boolean = false,
    var isAudioStarted: Boolean = false,
    var mode: Mode = Mode.WIFI
)

data class Result(
    val success: Boolean,
    val log: String
)


class ForegroundService : Service() {
    private val TAG = "MicService"
    private val scope = CoroutineScope(Dispatchers.Default)
    private val WAIT_PERIOD = 500L


    private inner class ServiceHandler(looper: Looper) : Handler(looper) {
        override fun handleMessage(msg: Message) {

            val commandData = CommandData.fromMessage(msg);

            when (Command.entries[msg.what]) {
                Command.StartStream -> startStream(commandData, msg.replyTo)
                Command.StopStream -> stopStream(msg.replyTo)
                Command.GetStatus -> getStatus(msg.replyTo)
            }

        }
    }

    private fun reply(replyTo: Messenger, resp: ResponseData) {
        replyTo.send(resp.toResponseMsg())
    }

    private lateinit var handlerThread: HandlerThread
    private lateinit var serviceLooper: Looper
    private lateinit var serviceHandler: ServiceHandler
    private lateinit var serviceMessenger: Messenger

    private var managerAudio: MicAudioManager? = null
    private var managerStream: MicStreamManager? = null


    private val states = ServiceStates()
    private lateinit var messageui: MessageUi


    override fun onCreate() {
        Log.d(TAG, "onCreate")
        // create message handler
        handlerThread = HandlerThread("MicServiceStart", Process.THREAD_PRIORITY_BACKGROUND)
        handlerThread.start()
        serviceLooper = handlerThread.looper
        serviceHandler = ServiceHandler(handlerThread.looper)
        serviceMessenger = Messenger(serviceHandler)

        // Create the NotificationChannel, but only on API 26+ because
        // the NotificationChannel class is new and not in the support library
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val name = getString(R.string.app_name)
            val importance = NotificationManager.IMPORTANCE_DEFAULT
            val channel = NotificationChannel(CHANNEL_ID, name, importance)
            // Register the channel with the system
            val notificationManager: NotificationManager =
                getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            notificationManager.createNotificationChannel(channel)
        }
        messageui = MessageUi(this)
    }

    override fun onBind(intent: Intent?): IBinder? {
        Log.d(TAG, "onBind")

        return serviceMessenger.binder
    }

    private var serviceShouldStop = false

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.d(TAG, "onStartCommand")
        serviceShouldStop = false
        return START_NOT_STICKY
    }

    override fun onUnbind(intent: Intent?): Boolean {
        super.onUnbind(intent)
        Log.d(TAG, "onUnbind")

        if (!states.isAudioStarted && !states.isStreamStarted) {
            // delay to handle reconfiguration
            // (Service is not destroy when the screen rotate)
            serviceShouldStop = true
            scope.launch {
                delay(3000L)
                if (serviceShouldStop)
                    stopService()
            }
        }
        return true
    }

    override fun onDestroy() {
        super.onDestroy()
        Log.d(TAG, "onDestroy")
        stopService()
    }

    private fun stopService() {
        Log.d(TAG, "stopService")
        managerAudio?.shutdown()
        managerAudio = null
        managerStream?.shutdown()
        managerStream = null
        serviceLooper.quitSafely()
        ignore { handlerThread.join(WAIT_PERIOD) }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            stopForeground(STOP_FOREGROUND_REMOVE)
        }
        stopSelf()
    }


    // start streaming
    private fun startStream(msg: CommandData, replyTo: Messenger) {

        if (!states.isAudioStarted) {
            if (!startAudio(msg, replyTo)) {
                return
            }
        }

        // check connection state
        if (states.isStreamStarted) {
            reply(replyTo, ResponseData(ServiceState.Connected, this.getString(R.string.stream_already_started)))
            return
        }

        Log.d(TAG, "startStream [start]")

        // try to start streaming
        managerStream?.shutdown()

        try {
            managerStream = MicStreamManager(applicationContext, scope, msg.mode!!, msg.ip, msg.port)
        } catch (e: IllegalArgumentException) {
            Log.d(TAG, "start stream with mode ${msg.mode!!.name} failed:\n${e.message}")

            reply(replyTo, ResponseData(ServiceState.Disconnected,  applicationContext.getString(R.string.error) + e.message))
            return
        }

        if (managerStream?.start(managerAudio!!.audioStream()) == true && managerStream?.isConnected() == true) {

            reply(replyTo, ResponseData(ServiceState.Connected,  applicationContext.getString(R.string.connected_device) + managerStream?.getInfo()))
            messageui.showMessage(applicationContext.getString(R.string.start_streaming))
            states.isStreamStarted = true
            Log.d(TAG, "startStream [connected]")
        } else {

            reply(replyTo, ResponseData(ServiceState.Disconnected,  applicationContext.getString(R.string.failed_to_connect)))

            managerStream?.shutdown()
            managerStream = null
        }
    }

    // stop streaming
    private fun stopStream(replyTo: Messenger) {
        Log.d(TAG, "stopStream")

        stopAudio(replyTo)

        managerStream?.shutdown()
        managerStream = null
        messageui.showMessage(applicationContext.getString(R.string.stop_streaming))
        states.isStreamStarted = false

        reply(replyTo, ResponseData(ServiceState.Disconnected, applicationContext.getString(R.string.device_disconnected)))
    }

    private fun isConnected(): ServiceState {
       return if (states.isStreamStarted) {
           ServiceState.Connected
        } else {
           ServiceState.Disconnected
        }
    }

    // start mic
    private fun startAudio(msg: CommandData, replyTo: Messenger): Boolean {
        // check audio state
        if (states.isAudioStarted) {
            reply(replyTo, ResponseData(msg = this.getString(R.string.microphone_already_started)))
            return true
        }

        Log.d(TAG, "startAudio [start]")

        // start audio recording
        managerAudio?.shutdown()
        try {
            managerAudio = MicAudioManager(
                ctx = applicationContext,
                scope = scope,
                sampleRate = msg.sampleRate!!.value,
                audioFormat = msg.audioFormat!!.value,
                channelCount = msg.channelCount!!.value,
            )
        } catch (e: IllegalArgumentException) {
            reply(replyTo, ResponseData(msg = application.getString(R.string.error) + e.message))
            return false
        }

        managerAudio?.start()

        // the id is not important here
        // we need to start in foreground to use the mic
        // but no need to specified a flag because we declared
        // the type in manifest
        startForeground(3, messageui.getNotification())

        messageui.showMessage(application.getString(R.string.start_recording))
        Log.d(TAG, "startAudio [recording]")
        states.isAudioStarted = true

        reply(replyTo, ResponseData(msg = application.getString(R.string.mic_start_recording)))

        return true
    }

    // stop mic
    private fun stopAudio(replyTo: Messenger) {
        Log.d(TAG, "stopAudio")
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            stopForeground(STOP_FOREGROUND_REMOVE)
        }
        managerAudio?.shutdown()
        managerAudio = null
        messageui.showMessage(application.getString(R.string.stop_recording))
        states.isAudioStarted = false

        reply(replyTo, ResponseData(msg = application.getString(R.string.recording_stopped)))
    }


    private fun getStatus(replyTo: Messenger) {
        Log.d(TAG, "getStatus")

        reply(replyTo, ResponseData(isConnected()))
    }
}