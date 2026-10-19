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
import android.os.Process
import android.util.Log
import androidx.core.content.ContextCompat
import io.github.teamclouday.androidMic.domain.service.AudioPacket
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.asCoroutineDispatcher
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.channelFlow
import kotlinx.coroutines.launch
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.concurrent.Executors


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

        // audio is sent in packets this long, the PC can't play it before a packet is complete
        const val CHUNK_MS = 5

        // the record buffer is kept drained so its size doesn't add latency,
        // a big one just survives longer scheduling or network hiccups without losing audio
        const val RECORD_BUFFER_MS = 500
    }

    private val recorder: AudioRecord
    private val effects: List<AudioEffect>
    private val chunkSize: Int
    private val buffer: ByteArray
    private val bufferFloat: FloatArray
    private val bufferFloatConvert: ByteBuffer
    private var streamJob: Job? = null

    // reading the mic on a normal priority thread adds scheduling jitter
    private val captureDispatcher = Executors.newSingleThreadExecutor { r ->
        Thread({
            Process.setThreadPriority(Process.THREAD_PRIORITY_URGENT_AUDIO)
            r.run()
        }, "AndroidMic capture")
    }.asCoroutineDispatcher()

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
        val minBufferSize = AudioRecord.getMinBufferSize(
            sampleRate,
            channelConfig,
            audioFormat,
        )

        require(minBufferSize != AudioRecord.ERROR && minBufferSize != AudioRecord.ERROR_BAD_VALUE) {
            "Microphone buffer size ($minBufferSize) is invalid\nAudio format is likely not supported"
        }

        val frameSize = channelCount * bytesPerSample(audioFormat)
        chunkSize = maxOf(1, sampleRate * CHUNK_MS / 1000) * frameSize
        val bufferFrames = maxOf(
            minBufferSize * 2 / frameSize,
            sampleRate * RECORD_BUFFER_MS / 1000
        )
        val bufferSize = bufferFrames * frameSize

        val isUnprocessed = isUnprocessedSource(audioSource)

        // the unprocessed source behaves like the default (processed) one when the device
        // doesn't support it, voice recognition is a better match in that case
        var source = audioSource
        if (isUnprocessed && !isUnprocessedSupported(ctx)) {
            Log.w(TAG, "unprocessed audio source not supported, using voice recognition")
            source = MediaRecorder.AudioSource.VOICE_RECOGNITION
            isUnprocessedFallback = true
        }

        // init recorder, with the minimum buffer if the device refuses a big one
        fun createRecorder(source: Int): AudioRecord {
            val rec = AudioRecord(source, sampleRate, channelConfig, audioFormat, bufferSize)
            if (rec.state == AudioRecord.STATE_INITIALIZED || bufferSize <= minBufferSize) {
                return rec
            }
            Log.w(TAG, "record buffer of $bufferSize bytes refused, using $minBufferSize")
            rec.release()
            return AudioRecord(source, sampleRate, channelConfig, audioFormat, minBufferSize)
        }

        var rec = createRecorder(source)

        if (rec.state != AudioRecord.STATE_INITIALIZED && !isUnprocessedFallback && isUnprocessed) {
            Log.w(TAG, "unprocessed audio source failed to initialize, using voice recognition")
            rec.release()
            rec = createRecorder(MediaRecorder.AudioSource.VOICE_RECOGNITION)
            isUnprocessedFallback = true
        }
        recorder = rec

        // check if recorder is initialized
        require(recorder.state == AudioRecord.STATE_INITIALIZED) {
            "Microphone recording failed to initialize"
        }

        effects = if (isUnprocessed) disablePreProcessing(recorder.audioSessionId) else emptyList()

        buffer = ByteArray(chunkSize)
        bufferFloat = FloatArray(chunkSize / 4) // float is 4 bytes
        bufferFloatConvert = ByteBuffer.allocate(chunkSize).order(ByteOrder.nativeOrder())

        Log.d(TAG, "record buffer: $bufferSize bytes, chunk: $chunkSize bytes")
    }

    // audio stream publisher
    fun audioStream(): Flow<AudioPacket> = channelFlow {
        // launch in scope so infinite loop will be canceled when scope exits
        streamJob = scope.launch(captureDispatcher) {
            while (true) {

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
                        packetBuffer = bufferFloatConvert.array().copyOf(readCount * 4)
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

                // keep the stream going while muted, so the PC doesn't have to buffer it again
                // and no stale audio piles up in the record buffer
                if (isMuted) {
                    packetBuffer.fill(silence(audioFormat))
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
        streamJob?.cancel()
        recorder.stop()
        recorder.release()
        effects.forEach { it.release() }
        captureDispatcher.close()
        Log.d(TAG, "shutdown")
    }

    private fun bytesPerSample(format: Int): Int {
        return when (format) {
            AudioFormat.ENCODING_PCM_8BIT -> 1
            AudioFormat.ENCODING_PCM_16BIT -> 2
            AudioFormat.ENCODING_PCM_24BIT_PACKED -> 3
            AudioFormat.ENCODING_PCM_32BIT, AudioFormat.ENCODING_PCM_FLOAT -> 4
            else -> 2
        }
    }

    // 8 bit pcm is unsigned
    private fun silence(format: Int): Byte {
        return if (format == AudioFormat.ENCODING_PCM_8BIT) 0x80.toByte() else 0
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