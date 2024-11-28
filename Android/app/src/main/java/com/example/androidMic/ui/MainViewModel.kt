package com.example.androidMic.ui

import android.os.Bundle
import android.os.Handler
import android.os.HandlerThread
import android.os.Looper
import android.os.Message
import android.os.Messenger
import android.os.Process
import android.util.Log
import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.example.androidMic.AndroidMicApp
import com.example.androidMic.AppPreferences
import com.example.androidMic.AudioFormat
import com.example.androidMic.ChannelCount
import com.example.androidMic.Dialogs
import com.example.androidMic.Modes
import com.example.androidMic.R
import com.example.androidMic.SampleRates
import com.example.androidMic.Themes
import com.example.androidMic.domain.service.Command
import com.example.androidMic.domain.service.Command.Companion.COMMAND_DISC_STREAM
import com.example.androidMic.domain.service.Command.Companion.COMMAND_GET_STATUS
import com.example.androidMic.ui.utils.UiHelper
import com.example.androidMic.utils.checkIp
import com.example.androidMic.utils.checkPort
import kotlinx.coroutines.launch


class MainViewModel : ViewModel() {

    private val TAG = "MainViewModel"


    val prefs: AppPreferences = AndroidMicApp.appModule.appPreferences
    val uiHelper: UiHelper = AndroidMicApp.appModule.uiHelper

    private var mService: Messenger? = null
    private var mBound = false

    lateinit var handlerThread: HandlerThread
    private lateinit var mMessenger: Messenger
    lateinit var mMessengerLooper: Looper
    private lateinit var mMessengerHandler: ReplyHandler


    val textLog = mutableStateOf("")

    val isAudioStarted = mutableStateOf(false)
    val isStreamStarted = mutableStateOf(false)
    val buttonConnectIsClickable = mutableStateOf(false)
    val switchAudioIsClickable = mutableStateOf(false)

    init {
        Log.d(TAG, "init")
    }

    private inner class ReplyHandler(looper: Looper) : Handler(looper) {
        override fun handleMessage(msg: Message) {
            when (msg.what) {
                COMMAND_DISC_STREAM -> handleDisconnect(msg)
                COMMAND_GET_STATUS -> handleGetStatus(msg)

                else -> handleResult(msg)
            }
        }
    }

    fun handlerServiceResponse() {
        handlerThread = HandlerThread("activity", Process.THREAD_PRIORITY_BACKGROUND)
        handlerThread.start()
        mMessengerLooper = handlerThread.looper
        mMessengerHandler = ReplyHandler(mMessengerLooper)
        mMessenger = Messenger(mMessengerHandler)
    }

    fun refreshAppVariables() {
        mBound = AndroidMicApp.mBound
        mService = AndroidMicApp.mService

        isAudioStarted.value = false
        isStreamStarted.value = false
        buttonConnectIsClickable.value = false
        switchAudioIsClickable.value = false
    }

    fun onConnectButton(): Dialogs? {
        if (!mBound) return null
        val reply = if (isStreamStarted.value) {
            Log.d(TAG, "onConnectButton: stop stream")
            Message.obtain(null, Command.COMMAND_STOP_STREAM)
        } else {
            val data = Bundle()

            val ip = prefs.ip.getBlocking()
            val port = prefs.port.getBlocking()
            val mode = prefs.mode.getBlocking()

            when (mode) {
                Modes.WIFI, Modes.UDP -> {
                    if (!checkIp(ip) || !checkPort(port)) {
                        uiHelper.makeToast(
                            uiHelper.getString(R.string.invalid_ip_port)
                        )
                        return Dialogs.IpPort
                    }
                    data.putString("IP", ip)
                    data.putInt("PORT", port.toInt())
                }

                Modes.USB -> {
                    if (!checkPort(port)) {
                        uiHelper.makeToast(
                            uiHelper.getString(R.string.invalid_port)
                        )

                        return Dialogs.IpPort
                    }
                    data.putInt("PORT", port.toInt())
                }

                else -> {}
            }

            data.putInt("MODE", mode.ordinal)

            Log.d(TAG, "onConnectButton: start stream")
            // lock button to avoid duplicate events
            buttonConnectIsClickable.value = false
            Message.obtain(null, Command.COMMAND_START_STREAM).apply {
                this.data = data
            }
        }

        reply.replyTo = mMessenger
        mService?.send(reply)

        return null
    }

    fun onAudioSwitch() {
        if (!mBound) return
        val reply = if (isAudioStarted.value) {
            Log.d(TAG, "onAudioSwitch: stop audio")
            Message.obtain(null, Command.COMMAND_STOP_AUDIO)
        } else {
            Log.d(TAG, "onAudioSwitch: start audio")
            Message.obtain(null, Command.COMMAND_START_AUDIO).apply {
                val data = Bundle()
                data.putInt("SAMPLE_RATE", prefs.sampleRate.getBlocking().value)
                data.putInt("CHANNEL_COUNT", prefs.channelCount.getBlocking().value)
                data.putInt("AUDIO_FORMAT", prefs.audioFormat.getBlocking().value)
                this.data = data
            }
        }
        // lock switch to avoid duplicate events
        switchAudioIsClickable.value = false
        reply.replyTo = mMessenger
        mService?.send(reply)
    }

    fun setIpPort(ip: String, port: String) {
        viewModelScope.launch {
            prefs.ip.update(ip)
            prefs.port.update(port)
        }
    }

    fun setMode(mode: Modes) {
        viewModelScope.launch {
            prefs.mode.update(mode)
        }
    }

    fun setSampleRate(sampleRate: SampleRates) {
        viewModelScope.launch {
            prefs.sampleRate.update(sampleRate)
        }
    }

    fun setChannelCount(channelCount: ChannelCount) {
        viewModelScope.launch {
            prefs.channelCount.update(channelCount)
        }
    }

    fun setAudioFormat(audioFormat: AudioFormat) {
        viewModelScope.launch {
            prefs.audioFormat.update(audioFormat)
        }
    }


    fun setTheme(theme: Themes) {
        viewModelScope.launch {
            prefs.theme.update(theme)
        }
    }

    fun setDynamicColor(dynamicColor: Boolean) {
        viewModelScope.launch {
            prefs.dynamicColor.update(dynamicColor)
        }
    }

    fun cleanLog() {
        textLog.value = ""
    }


    // ask foreground service for current status
    fun askForStatus() {
        if (!mBound) return
        val reply = Message.obtain(null, COMMAND_GET_STATUS)
        reply.replyTo = mMessenger
        mService?.send(reply)
    }


    // apply status to UI
    private fun handleGetStatus(msg: Message) {
        isStreamStarted.value = msg.data.getBoolean("isStreamStarted")
        isAudioStarted.value = msg.data.getBoolean("isAudioStarted")
        switchAudioIsClickable.value = true
        buttonConnectIsClickable.value = true
    }

    private fun handleResult(msg: Message) {
        val reply = msg.data.getString("reply")
        if (reply != null) addLogMessage(reply)

        val result = msg.data.getBoolean("result")

        when (msg.what) {
            Command.COMMAND_START_STREAM -> {
                isStreamStarted.value = result
                buttonConnectIsClickable.value = true
            }

            Command.COMMAND_STOP_STREAM -> {
                isStreamStarted.value = !result
                buttonConnectIsClickable.value = true

            }

            Command.COMMAND_START_AUDIO -> {
                isAudioStarted.value = result
                switchAudioIsClickable.value = true
            }

            Command.COMMAND_STOP_AUDIO -> {

                isAudioStarted.value = !result
                switchAudioIsClickable.value = true
            }
        }
    }

    private fun handleDisconnect(msg: Message) {
        Log.d(TAG, "handleDisconnect")
        val reply = msg.data.getString("reply")
        if (reply != null) addLogMessage(reply)
        isStreamStarted.value = false
    }

    // helper function to append log message to textview
    private fun addLogMessage(message: String) {
        textLog.value = textLog.value + message + "\n"
    }
}