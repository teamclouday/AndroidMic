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
import com.example.androidMic.Mode
import com.example.androidMic.R
import com.example.androidMic.SampleRates
import com.example.androidMic.Themes
import com.example.androidMic.domain.service.Command
import com.example.androidMic.domain.service.CommandData
import com.example.androidMic.domain.service.Response
import com.example.androidMic.domain.service.ResponseData
import com.example.androidMic.domain.service.ServiceState
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

    val isStreamStarted = mutableStateOf(false)
    val isButtonConnectClickable = mutableStateOf(false)

    init {
        Log.d(TAG, "init")
    }

    private inner class ReplyHandler(looper: Looper) : Handler(looper) {
        override fun handleMessage(msg: Message) {

            val data = ResponseData.fromMessage(msg);

            when (Response.entries[msg.what]) {
                Response.Standard -> {
                    data.state?.let {
                        isButtonConnectClickable.value = true
                        isStreamStarted.value = it == ServiceState.Connected
                    }

                    data.msg?.let {
                        addLogMessage(it)
                    }
                }
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

        isStreamStarted.value = false
        isButtonConnectClickable.value = false

        val msg = CommandData(Command.BindCheck).toCommandMsg()
        msg.replyTo = mMessenger
        mService?.send(msg)
    }

    fun onConnectButton(): Dialogs? {
        if (!mBound) return null
        val reply = if (isStreamStarted.value) {
            Log.d(TAG, "onConnectButton: stop stream")
            CommandData(Command.StopStream).toCommandMsg()
        } else {
            val ip = prefs.ip.getBlocking()
            val port = prefs.port.getBlocking()
            val mode = prefs.mode.getBlocking()

            val data = CommandData(
                command = Command.StartStream,
                sampleRate = prefs.sampleRate.getBlocking(),
                channelCount = prefs.channelCount.getBlocking(),
                audioFormat = prefs.audioFormat.getBlocking(),
                mode = mode
            )

            when (mode) {
                Mode.WIFI, Mode.UDP -> {
                    if (!checkIp(ip) || !checkPort(port)) {
                        uiHelper.makeToast(
                            uiHelper.getString(R.string.invalid_ip_port)
                        )
                        return Dialogs.IpPort
                    }
                    data.ip = ip
                    data.port =  port.toInt()
                }

                else -> {}
            }

            Log.d(TAG, "onConnectButton: start stream")
            // lock button to avoid duplicate events
            isButtonConnectClickable.value = false

            data.toCommandMsg()
        }

        reply.replyTo = mMessenger
        mService?.send(reply)

        return null
    }

    fun setIpPort(ip: String, port: String) {
        viewModelScope.launch {
            prefs.ip.update(ip)
            prefs.port.update(port)
        }
    }

    fun setMode(mode: Mode) {
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
        val reply = Message.obtain()
        reply.what = Command.GetStatus.ordinal
        reply.replyTo = mMessenger
        mService?.send(reply)
    }


    // helper function to append log message to textview
    private fun addLogMessage(message: String) {
        textLog.value = textLog.value + message + "\n"
    }
}