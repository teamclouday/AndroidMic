package io.github.teamclouday.androidMic.domain.audio

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.media.AudioFormat
import android.media.AudioManager
import android.media.AudioRecord
import android.media.MediaRecorder
import android.media.audiofx.AcousticEchoCanceler
import android.media.audiofx.AudioEffect
import android.media.audiofx.AutomaticGainControl
import android.media.audiofx.NoiseSuppressor
import android.os.Build
import android.util.Log
import androidx.core.content.ContextCompat
import io.github.teamclouday.androidMic.domain.service.AudioPacket
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.channelFlow
import kotlinx.coroutines.launch
import java.nio.ByteBuffer
import java.nio.ByteOrder


private const val TAG: String = "MicAM"

// manage microphone recording
class MicAudioManager(
    ctx: Context,
    val scope: CoroutineScope,
    val sampleRate: Int,
    val audioFormat: Int,
    val channelCount: Int,
    val audioSource: Int,
) {

    companion object {
        const val RECORD_DELAY_MS = 100L
    }

    private val recorder: AudioRecord
    private val effects: List<AudioEffect>
    private val bufferSize: Int
    private val buffer: ByteArray
    private val bufferFloat: FloatArray
    private val bufferFloatConvert: ByteBuffer
    private var streamJob: Job? = null

    private var isMuted = false

    // true when the unprocessed source was requested but the device can't provide it
    var isUnprocessedFallback = false
        private set

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
        val channelConfig =
            if (channelCount == 2) AudioFormat.CHANNEL_IN_STEREO else AudioFormat.CHANNEL_IN_MONO
        bufferSize = AudioRecord.getMinBufferSize(
            sampleRate,
            channelConfig,
            audioFormat,
        )

        require(bufferSize != AudioRecord.ERROR && bufferSize != AudioRecord.ERROR_BAD_VALUE) {
            "Microphone buffer size ($bufferSize) is invalid\nAudio format is likely not supported"
        }

        val isUnprocessed = isUnprocessedSource(audioSource)

        // the unprocessed source behaves like the default (processed) one when the device
        // doesn't support it, voice recognition is a better match in that case
        var source = audioSource
        if (isUnprocessed && !isUnprocessedSupported(ctx)) {
            Log.w(TAG, "unprocessed audio source not supported, using voice recognition")
            source = MediaRecorder.AudioSource.VOICE_RECOGNITION
            isUnprocessedFallback = true
        }

        // init recorder
        var rec = AudioRecord(source, sampleRate, channelConfig, audioFormat, bufferSize)

        if (rec.state != AudioRecord.STATE_INITIALIZED && !isUnprocessedFallback && isUnprocessed) {
            Log.w(TAG, "unprocessed audio source failed to initialize, using voice recognition")
            rec.release()
            rec = AudioRecord(
                MediaRecorder.AudioSource.VOICE_RECOGNITION,
                sampleRate,
                channelConfig,
                audioFormat,
                bufferSize,
            )
            isUnprocessedFallback = true
        }
        recorder = rec

        // check if recorder is initialized
        require(recorder.state == AudioRecord.STATE_INITIALIZED) {
            "Microphone recording failed to initialize"
        }

        effects = if (isUnprocessed) disablePreProcessing(recorder.audioSessionId) else emptyList()

        buffer = ByteArray(bufferSize)
        bufferFloat = FloatArray(bufferSize / 4) // float is 4 bytes
        bufferFloatConvert = ByteBuffer.allocate(bufferSize).order(ByteOrder.nativeOrder())
    }

    // audio stream publisher
    fun audioStream(): Flow<AudioPacket> = channelFlow {
        // launch in scope so infinite loop will be canceled when scope exits
        streamJob = scope.launch {
            while (true) {

                if (isMuted) {
                    delay(RECORD_DELAY_MS)
                    continue
                }

                if (recorder.state != AudioRecord.STATE_INITIALIZED || recorder.recordingState != AudioRecord.RECORDSTATE_RECORDING) {
                    delay(RECORD_DELAY_MS)
                    continue
                }

                val readCount: Int // number of samples read (for float) or number of bytes read (for int)
                val packetBuffer: ByteArray

                if (audioFormat == AudioFormat.ENCODING_PCM_FLOAT) {
                    readCount =
                        recorder.read(bufferFloat, 0, bufferFloat.size, AudioRecord.READ_BLOCKING)

                    if (readCount > 0) {
                        bufferFloatConvert.clear()
                        bufferFloatConvert.asFloatBuffer().put(bufferFloat, 0, readCount)
                        packetBuffer = bufferFloatConvert.array()
                    } else {
                        packetBuffer = ByteArray(0)
                    }
                } else {
                    readCount = recorder.read(buffer, 0, buffer.size, AudioRecord.READ_BLOCKING)

                    if (readCount > 0) {
                        packetBuffer = ByteArray(readCount)
                        buffer.copyInto(packetBuffer, 0, 0, readCount)
                    } else {
                        packetBuffer = ByteArray(0)
                    }
                }

                if (readCount <= 0) {
                    delay(RECORD_DELAY_MS)
                    continue
                }

                send(
                    AudioPacket(
                        buffer = packetBuffer,
                        sampleRate = sampleRate,
                        audioFormat = audioFormat,
                        channelCount = channelCount
                    )
                )
            }
        }

        awaitClose {
            streamJob?.cancel()
        }
    }

    fun mute() {
        isMuted = true
    }

    fun unmute() {
        isMuted = false
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
        effects.forEach { it.release() }
        streamJob?.cancel()
        Log.d(TAG, "shutdown")
    }

    private fun isUnprocessedSource(source: Int): Boolean {
        return Build.VERSION.SDK_INT >= Build.VERSION_CODES.N && source == MediaRecorder.AudioSource.UNPROCESSED
    }

    private fun isUnprocessedSupported(ctx: Context): Boolean {
        val audioManager = ctx.getSystemService(Context.AUDIO_SERVICE) as AudioManager
        return audioManager.getProperty(AudioManager.PROPERTY_SUPPORT_AUDIO_SOURCE_UNPROCESSED)
            .toBoolean()
    }

    // some vendors attach gain control, noise suppression or echo cancellation
    // to every input, explicitly turn them off to keep the signal raw
    private fun disablePreProcessing(sessionId: Int): List<AudioEffect> {
        val effects = mutableListOf<AudioEffect>()

        if (AutomaticGainControl.isAvailable()) {
            AutomaticGainControl.create(sessionId)?.let { effects.add(it) }
        }
        if (NoiseSuppressor.isAvailable()) {
            NoiseSuppressor.create(sessionId)?.let { effects.add(it) }
        }
        if (AcousticEchoCanceler.isAvailable()) {
            AcousticEchoCanceler.create(sessionId)?.let { effects.add(it) }
        }

        effects.forEach {
            try {
                it.enabled = false
            } catch (e: IllegalStateException) {
                Log.w(TAG, "failed to disable ${it.javaClass.simpleName}: ${e.message}")
            }
        }

        return effects
    }
}