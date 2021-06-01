package com.example.microphone

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.media.AudioFormat
import android.media.AudioManager
import android.media.AudioRecord
import android.media.MediaRecorder
import android.util.Log
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import java.nio.ByteBuffer
import java.nio.ByteOrder

// reference: https://dolby.io/blog/recording-audio-on-android-with-examples
// reference: https://twigstechtips.blogspot.com/2013/07/android-enable-noise-cancellation-in.html

class AudioHelper(mActivity: Context, private val mGlobalData : GlobalData)
{
    private val mLogTag : String = "AndroidMicAio"

    private val AUDIO_SOURCE : Int = MediaRecorder.AudioSource.VOICE_RECOGNITION
    private val SAMPLE_RATE : Int = 44100
    private val CHANNEL_CONFIG : Int = AudioFormat.CHANNEL_IN_MONO
    private val AUDIO_FORMAT : Int = AudioFormat.ENCODING_PCM_16BIT
    private val BUFFER_SIZE : Int = AudioRecord.getMinBufferSize(SAMPLE_RATE, CHANNEL_CONFIG, AUDIO_FORMAT)

    private var mRecorder : AudioRecord? = null
    private val mBuffer = ShortArray(BUFFER_SIZE)

    init
    {
        // check microphone
        require(mActivity.packageManager.hasSystemFeature(PackageManager.FEATURE_MICROPHONE)){
            "Microphone is not detected on this device"
        }
        require(ContextCompat.checkSelfPermission(mActivity, Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED){
            "Microphone recording is not permitted"
        }
        // setup noise suppression
        (mActivity.getSystemService(Context.AUDIO_SERVICE) as AudioManager).setParameters("noise_suppression=auto")
        // init recorder
        mRecorder = AudioRecord(AUDIO_SOURCE, SAMPLE_RATE, CHANNEL_CONFIG, AUDIO_FORMAT, BUFFER_SIZE)
        require(mRecorder?.state == AudioRecord.STATE_INITIALIZED){"Microphone not init properly"}
    }

    // store data in global object
    fun setData()
    {
        // read number of shorts
        val readShorts = mRecorder?.read(mBuffer, 0, BUFFER_SIZE) ?: return
        if(readShorts <= 0)
        {
            Thread.sleep(4)
            return
        }
        // convert shorts to byte array
        val convertShort = ShortArray(readShorts)
        convertShort.forEachIndexed { index, value ->
            convertShort[index] = mBuffer[index]
        }
        val newData = ByteArray(readShorts * 2)
        ByteBuffer.wrap(newData).order(ByteOrder.LITTLE_ENDIAN).asShortBuffer().put(convertShort)
        // store data
        mGlobalData.addData(newData)
        Log.d(mLogTag, "[setData] new data recorded (${newData.size} bytes)")
    }

    // start recording
    fun startMic()
    {
        mRecorder?.startRecording()
        Log.d(mLogTag, "startMic")
    }

    // stop recording
    fun stopMic()
    {
        mRecorder?.stop()
        Log.d(mLogTag, "stopMic")
    }

    // clean object
    fun clean()
    {
        mRecorder?.stop()
        mRecorder?.release()
        mRecorder = null
        Log.d(mLogTag, "clean")
    }
}