package com.example.microphone

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.os.*
import android.util.Log
import android.view.View
import android.view.WindowManager
import android.widget.*
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.appcompat.widget.SwitchCompat
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import com.example.microphone.service.ForegroundService
import com.example.microphone.streaming.MicStreamManager
import java.util.concurrent.atomic.AtomicBoolean

// helper function to ignore some exceptions
inline fun ignore(body: () -> Unit) {
    try {
        body()
    } catch (e: Exception) {
    }
}

class MainActivity : AppCompatActivity() {
    private val TAG: String = "MicMainActivity"
    private val WAIT_PERIOD = 500L

    private lateinit var mLogTextView: TextView
    private lateinit var mScroller: ScrollView

    data class States(
        val isStreamStarted: AtomicBoolean = AtomicBoolean(false),
        val isAudioStarted: AtomicBoolean = AtomicBoolean(false),
        val isIPInfoSet: AtomicBoolean = AtomicBoolean(false)
    )

    private val states = States()

    private var mService: Messenger? = null
    private var mBound = false

    private lateinit var handlerThread: HandlerThread
    private lateinit var mMessenger: Messenger
    private lateinit var mMessengerLooper: Looper
    private lateinit var mMessengerHandler: ReplyHandler

    private inner class ReplyHandler(looper: Looper) : Handler(looper) {
        override fun handleMessage(msg: Message) {
            when (msg.what) {
                ForegroundService.COMMAND_SET_IP_PORT -> handleSetIPInfo(msg)
                ForegroundService.COMMAND_DISC_STREAM -> handleDisconnect(msg)
                ForegroundService.COMMAND_GET_STATUS -> handleGetStatus(msg)
            }
            if (msg.what < ForegroundService.COMMAND_DISC_STREAM) {
                when (msg.data.getInt("Result")) {
                    0 -> handleFailure(msg)
                    1 -> handleSuccess(msg)
                }
            }
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        mLogTextView = findViewById(R.id.txt_log)
        // set action bar title
        supportActionBar?.setTitle(R.string.activity_name)
        // set scroll to focus down
        mScroller = findViewById(R.id.scrollView)
        // keeps screen on for this activity
        // otherwise when app enters background, the service will also stop
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
    }

    override fun onStart() {
        super.onStart()
        // check for audio permission
        if (ContextCompat.checkSelfPermission(
                this,
                Manifest.permission.RECORD_AUDIO
            ) != PackageManager.PERMISSION_GRANTED
        )
            ActivityCompat.requestPermissions(this, arrayOf(Manifest.permission.RECORD_AUDIO), 0)
        if (ContextCompat.checkSelfPermission(
                this,
                Manifest.permission.BLUETOOTH
            ) != PackageManager.PERMISSION_GRANTED
        )
            ActivityCompat.requestPermissions(this, arrayOf(Manifest.permission.BLUETOOTH), 0)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            if (ContextCompat.checkSelfPermission(
                    this,
                    Manifest.permission.BLUETOOTH_CONNECT
                ) != PackageManager.PERMISSION_GRANTED
            )
                ActivityCompat.requestPermissions(
                    this,
                    arrayOf(Manifest.permission.BLUETOOTH_CONNECT),
                    0
                )
        }
        // start handler thread
        handlerThread = HandlerThread("MicActivityStart", Process.THREAD_PRIORITY_BACKGROUND)
        handlerThread.start()
        mMessengerLooper = handlerThread.looper
        mMessengerHandler = ReplyHandler(handlerThread.looper)
        mMessenger = Messenger(mMessengerHandler)
        // get variable from application
        refreshAppVariables()
        // get status
        askForStatus()
    }

    override fun onStop() {
        super.onStop()
        mMessengerLooper.quitSafely()
        ignore { handlerThread.join(WAIT_PERIOD) }
    }

    override fun onNewIntent(intent: Intent?) {
        super.onNewIntent(intent)
        if (intent?.extras?.getBoolean("ForegroundServiceBound") == true) {
            // get variable from application
            refreshAppVariables()
            // get status
            askForStatus()
        }
    }

    private fun refreshAppVariables() {
        mService = (application as DefaultApp).mService
        mBound = (application as DefaultApp).mBound
    }

    // onclick for connect button
    fun onButtonConnect(view: View) {
        if (!mBound) {
            Toast.makeText(this, "Service not available", Toast.LENGTH_SHORT).show()
            return
        }
        // lock button to avoid duplicate events
        (view as Button).isClickable = false
        if (states.isStreamStarted.get()) {
            Log.d(TAG, "onButtonConnect: stop stream")
            val reply = Message.obtain(null, ForegroundService.COMMAND_STOP_STREAM)
            reply.replyTo = mMessenger
            mService?.send(reply)
        } else {
            Log.d(TAG, "onButtonConnect: start stream")
            val reply = Message.obtain(null, ForegroundService.COMMAND_START_STREAM)
            reply.replyTo = mMessenger
            mService?.send(reply)
        }
    }

    // on change for microphone switch
    fun onSwitchMic(view: View) {
        if (!mBound) {
            Toast.makeText(this, "Service not available", Toast.LENGTH_SHORT).show()
            return
        }
        // lock switch
        (view as SwitchCompat).isClickable = false
        if (states.isAudioStarted.get()) {
            Log.d(TAG, "onSwitchMic: stop audio")
            val reply = Message.obtain(null, ForegroundService.COMMAND_STOP_AUDIO)
            reply.replyTo = mMessenger
            mService?.send(reply)
        } else {
            Log.d(TAG, "onSwitchMic: start audio")
            val reply = Message.obtain(null, ForegroundService.COMMAND_START_AUDIO)
            reply.replyTo = mMessenger
            mService?.send(reply)
        }
    }

    // ask foreground service for current status
    private fun askForStatus() {
        if (!mBound) return
        val reply = Message.obtain(null, ForegroundService.COMMAND_GET_STATUS)
        reply.replyTo = mMessenger
        mService?.send(reply)
    }

    // apply status to UI
    fun handleGetStatus(msg: Message) {
        states.isStreamStarted.set(msg.data.getBoolean("isStreamStarted"))
        states.isAudioStarted.set(msg.data.getBoolean("isAudioStarted"))
        val connectButton = findViewById<Button>(R.id.connect)
        val audioSwitch = findViewById<SwitchCompat>(R.id.audio_switch)
        runOnUiThread {
            if (states.isStreamStarted.get()) {
                connectButton.setText(R.string.disconnect)
                connectButton.isClickable = true
            } else {
                connectButton.setText(R.string.connect)
                connectButton.isClickable = true
            }
            if (states.isAudioStarted.get()) {
                audioSwitch.isChecked = true
                audioSwitch.isClickable = true
            } else {
                audioSwitch.isChecked = false
                audioSwitch.isClickable = true
            }
        }
    }

    fun handleSuccess(msg: Message) {
        val reply = msg.data.getString("reply")
        if (reply != null) runOnUiThread { addLogMessage(reply) }
        val connectButton = findViewById<Button>(R.id.connect)
        val audioSwitch = findViewById<SwitchCompat>(R.id.audio_switch)
        when (msg.what) {
            ForegroundService.COMMAND_START_STREAM -> {
                // stream start success
                Log.d(TAG, "handleSuccess: COMMAND_START_STREAM")
                states.isStreamStarted.set(true)
                runOnUiThread {
                    connectButton.setText(R.string.disconnect)
                    connectButton.isClickable = true
                }
            }
            ForegroundService.COMMAND_STOP_STREAM -> {
                // stream stop success
                Log.d(TAG, "handleSuccess: COMMAND_STOP_STREAM")
                states.isStreamStarted.set(false)
                runOnUiThread {
                    connectButton.setText(R.string.connect)
                    connectButton.isClickable = true
                }
            }
            ForegroundService.COMMAND_START_AUDIO -> {
                // audio start success
                Log.d(TAG, "handleSuccess: COMMAND_START_AUDIO")
                states.isAudioStarted.set(true)
                runOnUiThread {
                    audioSwitch.isChecked = true
                    audioSwitch.isClickable = true
                }
            }
            ForegroundService.COMMAND_STOP_AUDIO -> {
                // audio stop success
                Log.d(TAG, "handleSuccess: COMMAND_STOP_AUDIO")
                states.isAudioStarted.set(false)
                runOnUiThread {
                    audioSwitch.isChecked = false
                    audioSwitch.isClickable = true
                }
            }
        }
    }

    fun handleFailure(msg: Message) {
        val reply = msg.data.getString("reply")
        if (reply != null) runOnUiThread { addLogMessage(reply) }
        val connectButton = findViewById<Button>(R.id.connect)
        val audioSwitch = findViewById<SwitchCompat>(R.id.audio_switch)
        when (msg.what) {
            ForegroundService.COMMAND_START_STREAM -> {
                // stream start fail
                Log.d(TAG, "handleFailure: COMMAND_START_STREAM")
                states.isStreamStarted.set(false)
                runOnUiThread {
                    connectButton.setText(R.string.connect)
                    connectButton.isClickable = true
                }
            }
            ForegroundService.COMMAND_STOP_STREAM -> {
                // stream stop fail
                Log.d(TAG, "handleFailure: COMMAND_STOP_STREAM")
                states.isStreamStarted.set(true)
                runOnUiThread {
                    connectButton.setText(R.string.disconnect)
                    connectButton.isClickable = true
                }
            }
            ForegroundService.COMMAND_START_AUDIO -> {
                // audio start fail
                Log.d(TAG, "handleFailure: COMMAND_START_AUDIO")
                states.isAudioStarted.set(false)
                runOnUiThread {
                    audioSwitch.isChecked = false
                    audioSwitch.isClickable = true
                }
            }
            ForegroundService.COMMAND_STOP_AUDIO -> {
                // audio stop fail
                Log.d(TAG, "handleFailure: COMMAND_STOP_AUDIO")
                states.isAudioStarted.set(true)
                runOnUiThread {
                    audioSwitch.isChecked = true
                    audioSwitch.isClickable = true
                }
            }
        }
    }

    fun handleDisconnect(msg: Message) {
        val reply = msg.data.getString("reply")
        if (reply != null) runOnUiThread { addLogMessage(reply) }
        runOnUiThread {
            findViewById<Button>(R.id.connect).setText(R.string.connect)
            findViewById<Button>(R.id.connect).isClickable = true
        }
        states.isStreamStarted.set(false)
    }

    fun handleSetIPInfo(msg: Message) {
        if (!msg.data.isEmpty) {
            if (msg.data.getInt("Result") == 0) {
                val reply = msg.data.getString("reply")
                if (reply != null) runOnUiThread { addLogMessage(reply) }
            }
        } else runOnUiThread { getIpAddress() }
    }

    // helper function to append log message to textview
    private fun addLogMessage(message: String) {
        mLogTextView.append(message + "\n")
        mScroller.fullScroll(View.FOCUS_DOWN)
    }

    private fun getIpAddress() {
        states.isIPInfoSet.set(false)

        val USER_SETTINGS_DEFAULT_IP = "DEFAULT_IP"
        val USER_SETTINGS_DEFAULT_PORT = "DEFAULT_PORT"
        val PREFERENCES_NAME = "AndroidMicUserSettings"
        val userSettings = applicationContext.getSharedPreferences(PREFERENCES_NAME, MODE_PRIVATE)
        var ip = userSettings.getString(USER_SETTINGS_DEFAULT_IP, MicStreamManager.DEFAULT_IP)
        var port = userSettings.getInt(USER_SETTINGS_DEFAULT_PORT, MicStreamManager.DEFAULT_PORT)

        val layout = layoutInflater.inflate(R.layout.alertdialog, null)
        layout.findViewById<EditText>(R.id.dialog_ip)
            .setText(ip)
        layout.findViewById<EditText>(R.id.dialog_port)
            .setText(port.toString())
        val builder = AlertDialog.Builder(this)
        builder.setTitle("PC Address IP/Port")
        builder.setView(layout)
        builder.setPositiveButton(
            "OK"
        ) { _, _ ->
            ip = layout.findViewById<EditText>(R.id.dialog_ip)
                .text.toString()
            port = try {
                layout.findViewById<EditText>(R.id.dialog_port)
                    .text.toString().toInt()
            } catch (e: NumberFormatException) {
                Log.d(TAG, "getIpAddress: invalid port")
                -1
            }
            val data = Bundle()
            data.putString("IP", ip)
            data.putInt("PORT", port)
            val reply = Message.obtain(null, ForegroundService.COMMAND_SET_IP_PORT)
            reply.data = data
            reply.replyTo = mMessenger
            if (!states.isIPInfoSet.get()) {
                mService?.send(reply)
                states.isIPInfoSet.set(true)
            }
            val editor = userSettings.edit()
            editor.putString(USER_SETTINGS_DEFAULT_IP, ip)
            editor.putInt(USER_SETTINGS_DEFAULT_PORT, port)
            editor.apply()
        }
        builder.setOnCancelListener {
            val data = Bundle()
            data.putString("IP", MicStreamManager.DEFAULT_IP)
            data.putInt("PORT", MicStreamManager.DEFAULT_PORT)
            val reply = Message.obtain(null, ForegroundService.COMMAND_SET_IP_PORT)
            reply.data = data
            reply.replyTo = mMessenger
            if (!states.isIPInfoSet.get()) {
                mService?.send(reply)
                states.isIPInfoSet.set(true)
            }
        }
        builder.setOnDismissListener {
            val data = Bundle()
            data.putString("IP", MicStreamManager.DEFAULT_IP)
            data.putInt("PORT", MicStreamManager.DEFAULT_PORT)
            val reply = Message.obtain(null, ForegroundService.COMMAND_SET_IP_PORT)
            reply.data = data
            reply.replyTo = mMessenger
            if (!states.isIPInfoSet.get()) {
                mService?.send(reply)
                states.isIPInfoSet.set(true)
            }
        }
        builder.show()
    }
}