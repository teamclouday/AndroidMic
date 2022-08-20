package com.example.microphone.service

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.*
import android.util.Log
import android.widget.Toast
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import com.example.microphone.MainActivity
import com.example.microphone.R
import com.example.microphone.audio.AudioBuffer
import com.example.microphone.audio.MicAudioManager
import com.example.microphone.ignore
import com.example.microphone.streaming.MicStreamManager
import kotlinx.coroutines.*
import java.net.InetSocketAddress
import java.util.concurrent.atomic.AtomicBoolean

class ForegroundService : Service() {
    private val TAG = "MicService"
    private val scope = CoroutineScope(Dispatchers.Default)
    private val WAIT_PERIOD = 500L

    companion object {
        const val COMMAND_START_STREAM = 1
        const val COMMAND_STOP_STREAM = 2
        const val COMMAND_START_AUDIO = 3
        const val COMMAND_STOP_AUDIO = 4
        const val COMMAND_DISC_STREAM = 10
        const val COMMAND_SET_IP_PORT = 20
        const val COMMAND_GET_STATUS = 30
    }

    private inner class ServiceHandler(looper: Looper) : Handler(looper) {
        override fun handleMessage(msg: Message) {
            when (msg.what) {
                COMMAND_START_STREAM -> startStream(msg)
                COMMAND_STOP_STREAM -> stopStream(msg)
                COMMAND_START_AUDIO -> startAudio(msg)
                COMMAND_STOP_AUDIO -> stopAudio(msg)
                COMMAND_SET_IP_PORT -> setIPInfo(msg)
                COMMAND_GET_STATUS -> getStatus(msg)
            }
        }
    }

    private lateinit var handlerThread: HandlerThread
    private lateinit var serviceLooper: Looper
    private lateinit var serviceHandler: ServiceHandler
    private lateinit var serviceMessenger: Messenger

    private var sharedBuffer = AudioBuffer()
    private var managerAudio: MicAudioManager? = null
    private var managerStream: MicStreamManager? = null
    private var jobStreamM: Job? = null
    private var jobAudioM: Job? = null

    data class States(
        val isStreamStarted: AtomicBoolean = AtomicBoolean(false),
        val streamShouldStop: AtomicBoolean = AtomicBoolean(false),
        val isAudioStarted: AtomicBoolean = AtomicBoolean(false),
        val audioShouldStop: AtomicBoolean = AtomicBoolean(false),
        val isIPInfoSet: AtomicBoolean = AtomicBoolean(false)
    )

    private val states = States()

    override fun onCreate() {
        // create message handler
        handlerThread = HandlerThread("MicServiceStart", Process.THREAD_PRIORITY_BACKGROUND)
        handlerThread.start()
        serviceLooper = handlerThread.looper
        serviceHandler = ServiceHandler(handlerThread.looper)
        serviceMessenger = Messenger(serviceHandler)
        // Create the NotificationChannel, but only on API 26+ because
        // the NotificationChannel class is new and not in the support library
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val name = getString(R.string.activity_name)
            val descriptionText = getString(R.string.notification_text)
            val importance = NotificationManager.IMPORTANCE_DEFAULT
            val channel = NotificationChannel("AndroidMic", name, importance).apply {
                description = descriptionText
            }
            // Register the channel with the system
            val notificationManager: NotificationManager =
                getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            notificationManager.createNotificationChannel(channel)
        }
        removeNotification()
    }

    override fun onBind(intent: Intent?): IBinder? {
        return serviceMessenger.binder
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.d(TAG, "onStartCommand")
        return START_NOT_STICKY
    }

    override fun onDestroy() {
        super.onDestroy()
        Log.d(TAG, "onDestroy")
        removeNotification()
        managerAudio?.shutdown()
        managerStream?.shutdown()
        states.streamShouldStop.set(true)
        states.audioShouldStop.set(true)
        runBlocking {
            delay(WAIT_PERIOD)
            if (jobStreamM?.isActive == true) jobStreamM?.cancel()
            if (jobAudioM?.isActive == true) jobAudioM?.cancel()
            jobStreamM?.join()
            jobAudioM?.join()
        }
        serviceLooper.quitSafely()
        ignore { handlerThread.join(WAIT_PERIOD) }
    }

    override fun onTaskRemoved(rootIntent: Intent?) {
        Log.d(TAG, "onTaskRemoved")
        stopSelf()

    }

    // start streaming
    private fun startStream(msg: Message) {
        val sender = msg.replyTo
        val replyData = Bundle()
        // check connection state
        if (states.isStreamStarted.get()) {
            replyData.putString("reply", "Stream already started")
            replySuccess(sender, replyData, COMMAND_START_STREAM)
            return
        } else if (jobStreamM?.isActive == true) {
            // avoid duplicate jobs
            replyFailed(sender, replyData, COMMAND_START_STREAM)
            return
        }
        Log.d(TAG, "startStream [start]")
        // try to start streaming
        jobStreamM = scope.launch {
            managerStream?.shutdown()
            managerStream = MicStreamManager(applicationContext)
            try {
                managerStream?.initialize()
            } catch (e: IllegalArgumentException) {
                replyData.putString("reply", "Error:\n" + e.message)
                replyFailed(sender, replyData, COMMAND_START_STREAM)
                cancel()
            }
            states.streamShouldStop.set(false)
            if (managerStream?.needInitIP() == true) {
                // ask UI to set IP
                states.isIPInfoSet.set(false)
                val askIP = Message()
                askIP.data = null
                askIP.what = COMMAND_SET_IP_PORT
                askIP.replyTo = serviceMessenger
                sender.send(askIP)
                // wait
                while (!states.streamShouldStop.get() && !states.isIPInfoSet.get())
                    delay(WAIT_PERIOD)
            }
            sharedBuffer.reset()
            showMessage("Starting streaming")
            if (managerStream?.start() == true && managerStream?.isConnected() == true) {
                showMessage("Device connected")
                replyData.putString(
                    "reply",
                    "Connected Device\n${managerStream?.getInfo()}"
                )
                replySuccess(sender, replyData, COMMAND_START_STREAM)
                Log.d(TAG, "startStream [connected]")
            } else {
                replyData.putString("reply", "Failed to connect")
                replyFailed(sender, replyData, COMMAND_START_STREAM)
                managerStream?.shutdown()
                cancel()
            }
            states.isStreamStarted.set(true)
            states.streamShouldStop.set(false)
            while (!states.streamShouldStop.get()) {
                if (managerStream?.isConnected() == true) {
                    managerStream?.stream(sharedBuffer)
                    delay(MicStreamManager.STREAM_DELAY)
                } else {
                    replyData.putString("reply", "Device disconnected")
                    showMessage("Device disconnected")
                    replyFailed(sender, replyData, COMMAND_DISC_STREAM)
                    break
                }
            }
            states.isStreamStarted.set(false)
        }
    }

    // stop streaming
    private fun stopStream(msg: Message) {
        Log.d(TAG, "stopStream")
        val sender = msg.replyTo
        val replyData = Bundle()
        runBlocking {
            states.streamShouldStop.set(true)
            managerStream?.shutdown()
            delay(WAIT_PERIOD)
            jobStreamM?.cancel()
        }
        managerStream = null
        jobStreamM = null
        showMessage("Device disconnected")
        states.isStreamStarted.set(false)
        replySuccess(sender, replyData, COMMAND_STOP_STREAM)
    }

    // start mic
    private fun startAudio(msg: Message) {
        val sender = msg.replyTo
        val replyData = Bundle()
        // check audio state
        if (states.isAudioStarted.get()) {
            replyData.putString("reply", "Microphone already started")
            replySuccess(sender, replyData, COMMAND_START_AUDIO)
            return
        } else if (jobAudioM?.isActive == true) {
            // avoid duplicate jobs
            replyFailed(sender, replyData, COMMAND_START_AUDIO)
            return
        }
        Log.d(TAG, "startAudio [start]")
        // start audio recording
        jobAudioM = scope.launch {
            managerAudio?.shutdown()
            managerAudio = try {
                MicAudioManager(applicationContext)
            } catch (e: IllegalArgumentException) {
                replyData.putString("reply", "Error:\n" + e.message)
                replyFailed(sender, replyData, COMMAND_START_AUDIO)
                cancel()
                null
            }
            // start recording
            sharedBuffer.reset()
            managerAudio?.start()
            showNotification()
            showMessage("Recording started")
            replyData.putString("reply", "Microphone starts recording")
            replySuccess(sender, replyData, COMMAND_START_AUDIO)
            Log.d(TAG, "startAudio [recording]")
            // record into buffer
            states.isAudioStarted.set(true)
            states.audioShouldStop.set(false)
            while (!states.audioShouldStop.get()) {
                managerAudio?.record(sharedBuffer)
                delay(MicAudioManager.RECORD_DELAY)
            }
            states.isAudioStarted.set(false)
            removeNotification()
        }
    }

    // stop mic
    private fun stopAudio(msg: Message) {
        Log.d(TAG, "stopAudio")
        val sender = msg.replyTo
        val replyData = Bundle()
        runBlocking {
            states.audioShouldStop.set(true)
            managerAudio?.shutdown()
            delay(WAIT_PERIOD)
            jobAudioM?.cancel()
        }
        managerAudio = null
        jobAudioM = null
        showMessage("Recording stopped")
        removeNotification()
        states.isAudioStarted.set(false)
        replySuccess(sender, replyData, COMMAND_STOP_AUDIO)
    }

    private fun setIPInfo(msg: Message) {
        val sender = msg.replyTo
        val replyData = Bundle()
        val ip = msg.data.getString("IP") ?: ""
        val port = msg.data.getInt("PORT")
        val address = try {
            InetSocketAddress(ip, port)
        } catch (e: IllegalArgumentException) {
            Log.d(TAG, "setIPInfo: ${e.message}")
            replyData.putString("reply", "Invalid IP Address ${ip}:${port}")
            replyFailed(sender, replyData, COMMAND_SET_IP_PORT)
            return
        }
        managerStream?.setIPInfo(address)
        states.isIPInfoSet.set(true)
        replySuccess(sender, replyData, COMMAND_SET_IP_PORT)
    }

    private fun getStatus(msg: Message) {
        val sender = msg.replyTo
        val replyData = Bundle()
        replyData.putBoolean("isStreamStarted", states.isStreamStarted.get())
        replyData.putBoolean("isAudioStarted", states.isAudioStarted.get())
        val reply = Message()
        reply.data = replyData
        reply.what = COMMAND_GET_STATUS
        reply.replyTo = serviceMessenger
        sender.send(reply)
    }

    // reply failed message
    private fun replyFailed(sender: Messenger, data: Bundle, what: Int) {
        data.putInt("Result", 0) // 0 for failure
        val reply = Message()
        reply.data = data
        reply.what = what
        reply.replyTo = serviceMessenger
        sender.send(reply)
    }

    // reply success message
    private fun replySuccess(sender: Messenger, data: Bundle, what: Int) {
        data.putInt("Result", 1) // 1 for success
        val reply = Message()
        reply.data = data
        reply.what = what
        reply.replyTo = serviceMessenger
        sender.send(reply)
    }

    // show message on UI
    private fun showMessage(message: String) {
        val ctx = this
        CoroutineScope(Dispatchers.Main).launch {
            Toast.makeText(ctx, message, Toast.LENGTH_SHORT).show()
        }
    }

    private fun showNotification() {
        val ctx = this
        CoroutineScope(Dispatchers.Main).launch {
            val onTap = Intent(ctx, MainActivity::class.java).apply {
                addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_BROUGHT_TO_FRONT)
            }
            val pendingIntent =
                PendingIntent.getActivity(ctx, 0, onTap, PendingIntent.FLAG_IMMUTABLE)
            val builder = NotificationCompat.Builder(ctx, "AndroidMic")
                .setSmallIcon(R.drawable.icon)
                .setContentTitle(getString(R.string.activity_name))
                .setContentText(getString(R.string.notification_text))
                .setPriority(NotificationCompat.PRIORITY_DEFAULT)
                .setContentIntent(pendingIntent)
                .setOngoing(true)
            with(NotificationManagerCompat.from(ctx))
            {
                notify(0, builder.build())
            }
        }
    }

    private fun removeNotification() {
        val ctx = this
        CoroutineScope(Dispatchers.Main).launch {
            with(NotificationManagerCompat.from(ctx))
            {
                cancelAll()
            }
        }
    }
}