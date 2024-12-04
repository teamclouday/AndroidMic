package com.example.androidMic.domain.audio

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.media.AudioDeviceInfo
import android.media.AudioFormat
import android.media.AudioManager
import android.media.AudioRecord
import android.media.MediaRecorder
import android.util.Log
import androidx.core.content.ContextCompat

// reference: https://dolby.io/blog/recording-audio-on-android-with-examples
// reference: https://twigstechtips.blogspot.com/2013/07/android-enable-noise-cancellation-in.html

// manage microphone recording
class MicAudioManager(
    ctx: Context,
    val sampleRate: Int,
    audioFormat: Int,
    channelCount: Int

) {
    private val TAG: String = "MicAM"

    companion object {
        const val RECORD_DELAY = 1L
        const val BUFFER_SIZE = 1024
    }

    private val recorder: AudioRecord

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
        recorder = AudioRecord(
            MediaRecorder.AudioSource.MIC,
            sampleRate,
            if (channelCount == 2) AudioFormat.CHANNEL_IN_STEREO else AudioFormat.CHANNEL_IN_MONO,
            audioFormat,
            BUFFER_SIZE,
        )
    }

    // store data in shared audio buffer
    suspend fun record(audioBuffer: AudioBuffer) {
        val region = audioBuffer.openWriteRegion(BUFFER_SIZE)
        val regionLen = region.first
        val regionOffset = region.second
        val bytesWritten = recorder.read(audioBuffer.buffer, regionOffset, regionLen)
        audioBuffer.closeWriteRegion(bytesWritten)
        if (bytesWritten > 0)
            Log.d(TAG, "[record] audio data recorded (${bytesWritten} bytes)")
    }

    // start recording
    fun start() {
        recorder.startRecording()
        Log.d(TAG, "start")
    }

    // stop recording
    fun stop() {
        recorder.stop()
        Log.d(TAG, "stop")
    }

    // shutdown manager
    // should not call any methods after calling
    fun shutdown() {
        recorder.stop()
        recorder.release()
        Log.d(TAG, "shutdown")
    }
}