package com.example.microphone

import android.Manifest
import android.app.NotificationChannel
import android.app.NotificationManager
import android.bluetooth.BluetoothAdapter
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.content.pm.PackageManager
import android.os.*
import androidx.appcompat.app.AppCompatActivity
import android.text.InputType
import android.util.Log
import android.view.View
import android.view.WindowManager
import android.widget.*
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.widget.SwitchCompat
import androidx.core.app.ActivityCompat
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat
import java.lang.Exception

// helper function to ignore some exceptions
inline fun ignore(body: () -> Unit)
{
    try {
        body()
    }catch (e : Exception){}
}

class MainActivity : AppCompatActivity()
{
    private val mLogTag : String = "MainActivityTag"
    private var defaultIP : String = "192.168."

    private lateinit var mLogTextView : TextView
    private lateinit var mScroller : ScrollView

    private var isBluetoothStarted = false
    private var isAudioStarted = false
    private var isUSBStarted = false
    private var isIPSet = false

    private var mService : Messenger? = null
    private var mBound = false

    private val mConnection = object : ServiceConnection
    {
        override fun onServiceConnected(name: ComponentName?, service: IBinder?)
        {
            mService = Messenger(service)
            mBound = true
        }

        override fun onServiceDisconnected(name: ComponentName?)
        {
            mService = null
            mBound = false
        }
    }

    private lateinit var handlerThread : HandlerThread
    private lateinit var mMessenger : Messenger
    private lateinit var mMessengerLooper : Looper
    private lateinit var mMessengerHandler : ReplyHandler

    private inner class ReplyHandler(looper: Looper) : Handler(looper)
    {
        override fun handleMessage(msg: Message)
        {
            when(msg.what)
            {
                BackgroundHelper.COMMAND_SET_IP   -> handleSetIP(msg)
                BackgroundHelper.COMMAND_DISC_BTH -> handleBthDisconnect(msg)
                BackgroundHelper.COMMAND_DISC_USB -> handleUSBDisconnect(msg)
            }
            if(msg.what < BackgroundHelper.COMMAND_DISC_BTH)
            {
                when(msg.data.getInt("Result"))
                {
                    0 -> handleFailure(msg)
                    1 -> handleSuccess(msg)
                }
            }
        }
    }

    override fun onCreate(savedInstanceState: Bundle?)
    {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        mLogTextView = findViewById(R.id.txt_log)
        // set action bar title
        supportActionBar?.setTitle(R.string.activity_name)
        // set scroll to focus down
        mScroller = findViewById(R.id.scrollView2)
        // set up notification
        if(Build.VERSION.SDK_INT >= Build.VERSION_CODES.O)
        {
            val name = "AndroidMic"
            val descriptionText = "Microphone is in use"
            val importance = NotificationManager.IMPORTANCE_DEFAULT
            val channel = NotificationChannel("AndroidMic", name, importance).apply {
                description = descriptionText
            }
            // Register the channel with the system
            val notificationManager: NotificationManager =
                getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            notificationManager.createNotificationChannel(channel)
        }

    }

    override fun onStart()
    {
        super.onStart()
        // check for audio permission
        if(ContextCompat.checkSelfPermission(this, Manifest.permission.RECORD_AUDIO) != PackageManager.PERMISSION_GRANTED)
            ActivityCompat.requestPermissions(this, arrayOf(Manifest.permission.RECORD_AUDIO), 0)
        // enable adapter
        if(!BluetoothAdapter.getDefaultAdapter().isEnabled)
        {
            val enableBthIntent = Intent(BluetoothAdapter.ACTION_REQUEST_ENABLE)
            startActivity(enableBthIntent)
        }
        // start handler thread
        handlerThread = HandlerThread("MainActivityStartArgs", Process.THREAD_PRIORITY_BACKGROUND)
        handlerThread.start()
        mMessengerLooper = handlerThread.looper
        mMessengerHandler = ReplyHandler(handlerThread.looper)
        mMessenger = Messenger(mMessengerHandler)
        // start and bind to service
        val intent = Intent(this, BackgroundHelper::class.java)
        startService(intent)
        bindService(intent, mConnection, Context.BIND_AUTO_CREATE)
    }

    override fun onStop() {
        super.onStop()
        stopService(intent)
        if(mBound)
        {
            unbindService(mConnection)
            mBound = false
        }
        mMessengerLooper.quitSafely()
        ignore { handlerThread.join(1000) }
    }

    // onclick for bluetooth button
    fun onButtonBluetooth(view : View)
    {
        (view as Button).isClickable = false // lock button
        if(isBluetoothStarted)
        {
            Log.d(mLogTag, "Stop Bluetooth")
            val reply = Message.obtain(null, BackgroundHelper.COMMAND_STOP_BLUETOOTH, 0, 0)
            reply.replyTo = mMessenger
            mService?.send(reply)
        }
        else
        {
            Log.d(mLogTag, "Start Bluetooth")
            val reply = Message.obtain(null, BackgroundHelper.COMMAND_START_BLUETOOTH, 0, 0)
            reply.replyTo = mMessenger
            mService?.send(reply)
        }
    }

    // onclick for USB button
    fun onButtonUSB(view : View)
    {
        (view as Button).isClickable = false // lock button
        if(isUSBStarted)
        {
            Log.d(mLogTag, "Stop USB")
            val reply = Message.obtain(null, BackgroundHelper.COMMAND_STOP_USB, 0, 0)
            reply.replyTo = mMessenger
            mService?.send(reply)
        }
        else
        {
            Log.d(mLogTag, "Start USB")
            val reply = Message.obtain(null, BackgroundHelper.COMMAND_START_USB, 0, 0)
            reply.replyTo = mMessenger
            mService?.send(reply)
        }
    }

    // on change for microphone switch
    // basically similar to bluetooth function
    fun onSwitchMic(view : View)
    {
        (view as SwitchCompat).isClickable = false // lock switch
        if(isAudioStarted)
        {
            Log.d(mLogTag, "Stop Audio")
            val reply = Message.obtain(null, BackgroundHelper.COMMAND_STOP_AUDIO, 0, 0)
            reply.replyTo = mMessenger
            mService?.send(reply)
        }
        else
        {
            Log.d(mLogTag, "Start Audio")
            val reply = Message.obtain(null, BackgroundHelper.COMMAND_START_AUDIO, 0, 0)
            reply.replyTo = mMessenger
            mService?.send(reply)
        }
    }

    fun handleSuccess(msg : Message)
    {
        val reply = msg.data.getString("reply")
        if(reply != null) runOnUiThread { addLogMessage(reply) }
        val bthButton = findViewById<Button>(R.id.bth_connect)
        val usbButton = findViewById<Button>(R.id.usb_connect)
        val aioSwitch = findViewById<SwitchCompat>(R.id.audio_switch)
        when(msg.what)
        {
            BackgroundHelper.COMMAND_START_BLUETOOTH -> {
                // bluetooth start success
                Log.d(mLogTag, "handleSuccess COMMAND_START_BLUETOOTH")
                isBluetoothStarted = true
                runOnUiThread {
                    bthButton.setText(R.string.turn_off_bth)
                    bthButton.isClickable = true
                }
            }
            BackgroundHelper.COMMAND_STOP_BLUETOOTH -> {
                // bluetooth stop success
                Log.d(mLogTag, "handleSuccess COMMAND_STOP_BLUETOOTH")
                isBluetoothStarted = false
                runOnUiThread {
                    bthButton.setText(R.string.turn_on_bth)
                    bthButton.isClickable = true
                }
            }
            BackgroundHelper.COMMAND_START_AUDIO -> {
                // audio start success
                Log.d(mLogTag, "handleSuccess COMMAND_START_AUDIO")
                isAudioStarted = true
                runOnUiThread {
                    showNotification()
                    aioSwitch.isChecked = true
                    aioSwitch.isClickable = true
                }
            }
            BackgroundHelper.COMMAND_STOP_AUDIO -> {
                // audio stop success
                Log.d(mLogTag, "handleSuccess COMMAND_STOP_AUDIO")
                isAudioStarted = false
                runOnUiThread {
                    removeNotification()
                    aioSwitch.isChecked = false
                    aioSwitch.isClickable = true
                }
            }
            BackgroundHelper.COMMAND_START_USB -> {
                // usb start success
                Log.d(mLogTag, "handleSuccess COMMAND_START_USB")
                isUSBStarted = true
                runOnUiThread {
                    usbButton.setText(R.string.turn_off_usb)
                    usbButton.isClickable = true
                }
            }
            BackgroundHelper.COMMAND_STOP_USB -> {
                // usb stop success
                Log.d(mLogTag, "handleSuccess COMMAND_STOP_USB")
                isUSBStarted = false
                runOnUiThread {
                    usbButton.setText(R.string.turn_on_usb)
                    usbButton.isClickable = true
                }
            }
        }
    }

    fun handleFailure(msg : Message)
    {
        val reply = msg.data.getString("reply")
        if(reply != null) runOnUiThread { addLogMessage(reply) }
        val bthButton = findViewById<Button>(R.id.bth_connect)
        val usbButton = findViewById<Button>(R.id.usb_connect)
        val aioSwitch = findViewById<SwitchCompat>(R.id.audio_switch)
        when(msg.what)
        {
            BackgroundHelper.COMMAND_START_BLUETOOTH -> {
                // bluetooth start fail
                Log.d(mLogTag, "handleSuccess COMMAND_START_BLUETOOTH")
                isBluetoothStarted = false
                runOnUiThread {
                    bthButton.setText(R.string.turn_on_bth)
                    bthButton.isClickable = true
                }
            }
            BackgroundHelper.COMMAND_STOP_BLUETOOTH -> {
                // bluetooth stop fail
                Log.d(mLogTag, "handleSuccess COMMAND_STOP_BLUETOOTH")
                isBluetoothStarted = true
                runOnUiThread {
                    bthButton.setText(R.string.turn_off_bth)
                    bthButton.isClickable = true
                }
            }
            BackgroundHelper.COMMAND_START_AUDIO -> {
                // audio start fail
                Log.d(mLogTag, "handleSuccess COMMAND_START_AUDIO")
                isAudioStarted = false
                runOnUiThread {
                    aioSwitch.isChecked = false
                    aioSwitch.isClickable = true
                }
            }
            BackgroundHelper.COMMAND_STOP_AUDIO -> {
                // audio stop fail
                Log.d(mLogTag, "handleSuccess COMMAND_STOP_AUDIO")
                isAudioStarted = true
                runOnUiThread {
                    aioSwitch.isChecked = true
                    aioSwitch.isClickable = true
                }
            }
            BackgroundHelper.COMMAND_START_USB -> {
                // usb start fail
                Log.d(mLogTag, "handleSuccess COMMAND_START_USB")
                isUSBStarted = false
                runOnUiThread {
                    usbButton.setText(R.string.turn_on_usb)
                    usbButton.isClickable = true
                }
            }
            BackgroundHelper.COMMAND_STOP_USB -> {
                // usb stop fail
                Log.d(mLogTag, "handleSuccess COMMAND_STOP_USB")
                isUSBStarted = true
                runOnUiThread {
                    usbButton.setText(R.string.turn_off_usb)
                    usbButton.isClickable = true
                }
            }
        }
    }

    fun handleBthDisconnect(msg : Message)
    {
        val reply = msg.data.getString("reply")
        if(reply != null) runOnUiThread { addLogMessage(reply) }
        runOnUiThread {
            findViewById<Button>(R.id.bth_connect).setText(R.string.turn_on_bth)
            findViewById<Button>(R.id.bth_connect).isClickable = true
        }
        isBluetoothStarted = false
    }

    fun handleUSBDisconnect(msg : Message)
    {
        val reply = msg.data.getString("reply")
        if(reply != null) runOnUiThread { addLogMessage(reply) }
        runOnUiThread {
            findViewById<Button>(R.id.usb_connect).setText(R.string.turn_on_usb)
            findViewById<Button>(R.id.usb_connect).isClickable = true
        }
        isUSBStarted = false
    }

    fun handleSetIP(msg : Message)
    {
        if(!msg.data.isEmpty)
        {
            if(msg.data.getInt("Result") == 0)
            {
                val reply = msg.data.getString("reply")
                if(reply != null) runOnUiThread { addLogMessage(reply) }
            }
        }
        else runOnUiThread { getIpAddress() }
    }

    // helper function to append log message to textview
    private fun addLogMessage(message : String)
    {
        mLogTextView.append(message + "\n")
        mScroller.fullScroll(View.FOCUS_DOWN)
    }

    private fun getIpAddress()
    {
        isIPSet = false
        val builder = AlertDialog.Builder(this)
        builder.setTitle("PC IP Address")
        val input = EditText(this)
        input.setText(defaultIP)
        input.inputType = InputType.TYPE_CLASS_PHONE
        builder.setView(input)
        builder.setPositiveButton("OK"
        ) { dialog, which ->
            defaultIP = input.text.toString()
            val data = Bundle()
            data.putString("IP", defaultIP)
            val reply = Message.obtain(null, BackgroundHelper.COMMAND_SET_IP, 0, 0)
            reply.data = data
            reply.replyTo = mMessenger
            if(!isIPSet)
            {
                mService?.send(reply)
                isIPSet = true
            }
        }
        builder.setOnCancelListener {
            val data = Bundle()
            data.putString("IP", defaultIP)
            val reply = Message.obtain(null, BackgroundHelper.COMMAND_SET_IP, 0, 0)
            reply.data = data
            reply.replyTo = mMessenger
            if(!isIPSet)
            {
                mService?.send(reply)
                isIPSet = true
            }
        }
        builder.setOnDismissListener {
            val data = Bundle()
            data.putString("IP", defaultIP)
            val reply = Message.obtain(null, BackgroundHelper.COMMAND_SET_IP, 0, 0)
            reply.data = data
            reply.replyTo = mMessenger
            if(!isIPSet)
            {
                mService?.send(reply)
                isIPSet = true
            }
        }
        builder.show()
    }

    private fun showNotification()
    {
        val builder = NotificationCompat.Builder(this, "AndroidMic")
            .setSmallIcon(R.drawable.icon)
            .setContentTitle("AndroidMic")
            .setContentText("Microphone is in use")
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .setOngoing(true)
        with(NotificationManagerCompat.from(this))
        {
            notify(0, builder.build())
        }
    }

    private fun removeNotification()
    {
        with(NotificationManagerCompat.from(this))
        {
            cancel(0)
        }
    }
}