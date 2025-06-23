package io.github.teamclouday.AndroidMic.domain.service

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Intent
import android.os.Build
import android.os.Handler
import android.os.HandlerThread
import android.os.IBinder
import android.os.Looper
import android.os.Message
import android.os.Messenger
import android.os.Process
import android.util.Log
import io.github.teamclouday.AndroidMic.Mode
import io.github.teamclouday.AndroidMic.R
import io.github.teamclouday.AndroidMic.domain.audio.MicAudioManager
import io.github.teamclouday.AndroidMic.domain.streaming.MicStreamManager
import io.github.teamclouday.AndroidMic.utils.ignore
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch


data class ServiceStates(
    var isStreamStarted: Boolean = false,
    var isAudioStarted: Boolean = false,
    var mode: Mode = Mode.WIFI
)

private const val TAG = "MicService"
private const val WAIT_PERIOD = 500L

class ForegroundService : Service() {
    private val scope = CoroutineScope(Dispatchers.Default)

    private inner class ServiceHandler(looper: Looper) : Handler(looper) {
        override fun handleMessage(msg: Message) {

            val commandData = CommandData.fromMessage(msg)

            when (Command.entries[msg.what]) {
                Command.StartStream -> startStream(commandData, msg.replyTo)
                Command.StopStream -> stopStream(msg.replyTo)
                Command.GetStatus -> getStatus(msg.replyTo)
                Command.BindCheck -> {
                    uiMessenger = msg.replyTo
                }
            }

        }
    }

    private fun reply(replyTo: Messenger?, resp: ResponseData) {
        replyTo?.send(resp.toResponseMsg())
    }

    private lateinit var handlerThread: HandlerThread
    private lateinit var serviceLooper: Looper
    private lateinit var serviceHandler: ServiceHandler
    private lateinit var serviceMessenger: Messenger

    private var managerAudio: MicAudioManager? = null
    private var managerStream: MicStreamManager? = null


    private val states = ServiceStates()
    private lateinit var messageui: MessageUi

    // This field is true if the UI is running
    private var isBind = false
    private var uiMessenger: Messenger? = null


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
                getSystemService(NOTIFICATION_SERVICE) as NotificationManager
            notificationManager.createNotificationChannel(channel)
        }
        messageui = MessageUi(this)
    }

    override fun onBind(intent: Intent?): IBinder? {
        Log.d(TAG, "onBind")
        isBind = true

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
        isBind = false
        uiMessenger = null

        if (!states.isStreamStarted) {
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


    private fun showMessage(msg: String) {
        if (!isBind) {
            messageui.showMessage(msg)
        }
    }

    // start streaming
    private fun startStream(msg: CommandData, replyTo: Messenger) {
        // check connection state
        if (states.isStreamStarted) {
            reply(
                replyTo,
                ResponseData(
                    ServiceState.Connected,
                    this.getString(R.string.stream_already_started)
                )
            )
            return
        }
        shutdownStream()
        shutdownAudio()

        Log.d(TAG, "startStream [start]")

        try {
            managerStream =
                MicStreamManager(applicationContext, scope, msg.mode!!, msg.ip, msg.port)
        } catch (e: IllegalArgumentException) {
            Log.d(TAG, "start stream with mode ${msg.mode!!.name} failed:\n${e.message}")

            reply(
                replyTo,
                ResponseData(
                    ServiceState.Disconnected,
                    applicationContext.getString(R.string.error) + e.message
                )
            )
            return
        }

        if (managerStream?.connect() != true || managerStream?.isConnected() != true
        ) {

            reply(
                replyTo,
                ResponseData(
                    ServiceState.Disconnected,
                    applicationContext.getString(R.string.failed_to_connect)
                )
            )
            shutdownStream()
            return
        }


        if (!startAudio(msg, replyTo)) {
            shutdownStream()
            shutdownAudio()
            return
        }

        managerStream?.start(
            managerAudio!!.audioStream(),
            serviceMessenger
        )

        states.isStreamStarted = true
        Log.d(TAG, "startStream [connected]")

        reply(
            replyTo,
            ResponseData(
                ServiceState.Connected,
                applicationContext.getString(R.string.connected_device) + managerStream?.getInfo()
            )
        )

    }

    // stop streaming
    fun stopStream(replyTo: Messenger?) {
        Log.d(TAG, "stopStream")

        stopAudio(replyTo)

        shutdownStream()

        reply(
            uiMessenger,
            ResponseData(
                ServiceState.Disconnected,
                applicationContext.getString(R.string.device_disconnected)
            )
        )

        if (!isBind) {
            stopService()
        }
    }

    private fun shutdownStream() {
        managerStream?.shutdown()
        managerStream = null
        states.isStreamStarted = false
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

        Log.d(TAG, "startAudio [recording]")
        states.isAudioStarted = true

        reply(replyTo, ResponseData(msg = application.getString(R.string.mic_start_recording)))

        return true
    }

    // stop mic
    private fun stopAudio(replyTo: Messenger?) {
        Log.d(TAG, "stopAudio")
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            stopForeground(STOP_FOREGROUND_REMOVE)
        }
        shutdownAudio()
        reply(replyTo, ResponseData(msg = application.getString(R.string.recording_stopped)))
    }


    private fun shutdownAudio() {
        managerAudio?.shutdown()
        managerAudio = null
        states.isAudioStarted = false
    }


    private fun getStatus(replyTo: Messenger) {
        Log.d(TAG, "getStatus")

        reply(replyTo, ResponseData(isConnected()))
    }
}