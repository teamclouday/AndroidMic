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
    }

    private val recorder: AudioRecord
    private val bufferSize: Int

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

        // get minimum buffer size
        val channelConfig = if (channelCount == 2) AudioFormat.CHANNEL_IN_STEREO else AudioFormat.CHANNEL_IN_MONO
        bufferSize = AudioRecord.getMinBufferSize(
            sampleRate,
            channelConfig,
            audioFormat,
        )

        require(bufferSize != AudioRecord.ERROR && bufferSize != AudioRecord.ERROR_BAD_VALUE) {
            "Microphone buffer size ($bufferSize) is invalid\nAudio format is likely not supported"
        }

        // init recorder
        recorder = AudioRecord(
            MediaRecorder.AudioSource.MIC,
            sampleRate,
            channelConfig,
            audioFormat,
            bufferSize,
        )

        // check if recorder is intiialized
        require(recorder.state == AudioRecord.STATE_INITIALIZED) {
            "Microphone recording failed to initialize"
        }
    }

    // store data in shared audio buffer
    suspend fun record(audioBuffer: AudioBuffer) {
        val region = audioBuffer.openWriteRegion(bufferSize)
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