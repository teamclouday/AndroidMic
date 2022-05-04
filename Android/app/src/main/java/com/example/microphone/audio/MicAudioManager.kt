package com.example.microphone.audio

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.media.AudioFormat
import android.media.AudioManager
import android.media.AudioRecord
import android.media.MediaRecorder
import android.os.Build
import android.util.Log
import androidx.core.content.ContextCompat
import kotlinx.coroutines.delay
import java.nio.ByteBuffer
import java.nio.ByteOrder

// reference: https://dolby.io/blog/recording-audio-on-android-with-examples
// reference: https://twigstechtips.blogspot.com/2013/07/android-enable-noise-cancellation-in.html

// manage microphone recording
class MicAudioManager(ctx : Context) {
    private val TAG : String = "MicAM"

    companion object
    {
        const val RECORD_DELAY = 1L
        const val AUDIO_SOURCE : Int = MediaRecorder.AudioSource.MIC
        const val SAMPLE_RATE : Int = 16000
        const val CHANNEL_CONFIG : Int = AudioFormat.CHANNEL_IN_MONO
        const val AUDIO_FORMAT : Int = AudioFormat.ENCODING_PCM_16BIT
        val BUFFER_SIZE : Int = AudioRecord.getMinBufferSize(SAMPLE_RATE, CHANNEL_CONFIG, AUDIO_FORMAT)
    }

    private var recorder : AudioRecord? = null
    private val buffer = ShortArray(BUFFER_SIZE)

    init
    {
        // check microphone
        require(ctx.packageManager.hasSystemFeature(PackageManager.FEATURE_MICROPHONE)){
            "Microphone is not detected on this device"
        }
        require(ContextCompat.checkSelfPermission(ctx, Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED){
            "Microphone recording is not permitted"
        }
        // setup noise suppression
        (ctx.getSystemService(Context.AUDIO_SERVICE) as AudioManager).setParameters("noise_suppression=auto")
        // init recorder
        recorder = AudioRecord(AUDIO_SOURCE, SAMPLE_RATE, CHANNEL_CONFIG, AUDIO_FORMAT, BUFFER_SIZE)
        require(recorder?.state == AudioRecord.STATE_INITIALIZED){"Microphone not init properly"}
    }

    // store data in shared audio buffer
    suspend fun record(audioBuffer : AudioBuffer)
    {
        // read number of shorts
        val size = recorder?.read(buffer, 0, BUFFER_SIZE, AudioRecord.READ_BLOCKING) ?: return
        if(size <= 0)
        {
            delay(4)
            return
        }
        // create bytearray
        val data = ByteArray(size * 2)
        val dataWrapper = ByteBuffer.wrap(data)
            .order(ByteOrder.LITTLE_ENDIAN)
            .asShortBuffer()
        for(i in 0 until size)
            dataWrapper.put(buffer[i])
        // store data
        audioBuffer.push(data)
        Log.d(TAG, "[record] audio data recorded (${data.size} bytes)")
    }

    // start recording
    fun start()
    {
        recorder?.startRecording()
        Log.d(TAG, "start")
    }

    // stop recording
    fun stop()
    {
        recorder?.stop()
        Log.d(TAG, "stop")
    }

    // shutdown manager
    fun shutdown()
    {
        recorder?.stop()
        recorder?.release()
        recorder = null
        Log.d(TAG, "shutdown")
    }
}