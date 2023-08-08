package com.example.androidMic.domain.audio

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.media.AudioDeviceInfo
import android.media.AudioManager
import android.util.Log
import androidx.core.content.ContextCompat

// reference: https://dolby.io/blog/recording-audio-on-android-with-examples
// reference: https://twigstechtips.blogspot.com/2013/07/android-enable-noise-cancellation-in.html

// manage microphone recording
class MicAudioManager(ctx: Context, val sampleRate: Int) {
    private val TAG: String = "MicAM"

    companion object {
        const val RECORD_DELAY = 1L
        const val BUFFER_SIZE = 1024
        const val BUFFER_COUNT = 2
    }

    private val recorder: OboeRecorder

    init {
        // check microphone
        require(ctx.packageManager.hasSystemFeature(PackageManager.FEATURE_MICROPHONE)) {
            "Microphone is not detected on this device"
        }
        require(
            ContextCompat.checkSelfPermission(
                ctx,
                Manifest.permission.RECORD_AUDIO
            ) == PackageManager.PERMISSION_GRANTED
        ) {
            "Microphone recording is not permitted"
        }
        // find audio device
        val am = ctx.getSystemService(Context.AUDIO_SERVICE) as AudioManager
        val devices = am.getDevices(AudioManager.GET_DEVICES_INPUTS)
        require(devices.isNotEmpty()) {
            "No valid microphone device"
        }
        var selectedDevice = devices[0]
        for (device in devices) {
            if (device.type == AudioDeviceInfo.TYPE_BUILTIN_MIC) {
                selectedDevice = device
                break
            }
        }
        Log.d(TAG, "[init] selected input device ${selectedDevice.productName}")
        // init recorder
        recorder = OboeRecorder(selectedDevice.id, sampleRate, BUFFER_SIZE * BUFFER_COUNT)
    }

    // store data in shared audio buffer
    suspend fun record(audioBuffer: AudioBuffer) {
        val region = audioBuffer.openWriteRegion(BUFFER_SIZE)
        val regionLen = region.first
        val regionOffset = region.second
        val bytesWritten = recorder.readToBytes(audioBuffer.buffer, regionOffset, regionLen)
        audioBuffer.closeWriteRegion(bytesWritten)
        if (bytesWritten > 0)
            Log.d(TAG, "[record] audio data recorded (${bytesWritten} bytes)")
    }

    // start recording
    fun start() {
        recorder.startRecord()
        Log.d(TAG, "start")
    }

    // stop recording
    fun stop() {
        recorder.stopRecord()
        Log.d(TAG, "stop")
    }

    // shutdown manager
    // should not call any methods after calling
    fun shutdown() {
        recorder.stopRecord()
        Log.d(TAG, "shutdown")
    }
}