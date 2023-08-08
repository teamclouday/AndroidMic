package com.example.androidMic.domain.audio

class OboeRecorder(
    deviceId: Int,
    val sampleRate: Int,
    audioFormat: Int,
    channelCount: Int,
    bufferSize: Int
) {
    companion object {
        init {
            System.loadLibrary("microphone")
        }
    }

    init {
        setDeviceId(deviceId)
        setSampleRate(sampleRate)
        setChannelCount(channelCount)
        setAudioFormat(audioFormat)
        setBufferSizeInFrames(bufferSize)
    }

    fun startRecord() {
        startRecordingInternal()
    }

    fun stopRecord() {
        stopRecordingInternal()
    }

    fun readToBytes(buffer: ByteArray, offset: Int, len: Int, blocking: Boolean = false): Int {
        return if (blocking)
            readInternalBytesBlocking(buffer, offset, len)
        else
            readInternalBytes(buffer, offset, len)
    }

    fun readToShort(buffer: ShortArray, offset: Int, len: Int, blocking: Boolean = false): Int {
        return if (blocking)
            readInternalShortsBlocking(buffer, offset, len)
        else
            readInternalShorts(buffer, offset, len)
    }

    private external fun setDeviceId(deviceId: Int)

    private external fun setSampleRate(sampleRate: Int)
    private external fun setChannelCount(channelCount: Int)
    private external fun setAudioFormat(audioFormat: Int)

    private external fun setBufferSizeInFrames(frames: Int)

    private external fun startRecordingInternal()

    private external fun stopRecordingInternal()

    private external fun readInternalBytes(buffer: ByteArray, offset: Int, len: Int): Int

    private external fun readInternalShorts(buffer: ShortArray, offset: Int, len: Int): Int

    private external fun readInternalBytesBlocking(buffer: ByteArray, offset: Int, len: Int): Int

    private external fun readInternalShortsBlocking(buffer: ShortArray, offset: Int, len: Int): Int
}