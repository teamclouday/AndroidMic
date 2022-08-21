package com.example.microphone.audio

class OboeRecorder(
    val deviceId: Int,
    val sampleRate: Int,
    val bufferSize: Int
) {
    companion object {
        init {
            System.loadLibrary("microphone")
        }
    }

    init {
        setDeviceId(deviceId)
        setSampleRate(sampleRate)
        setBufferSizeInFrames(bufferSize)
    }

    fun startRecord() {
        startRecordingInternal()
    }

    fun stopRecord() {
        stopRecordingInternal()
    }

    fun read(buffer: ShortArray, numShorts: Int): Int {
        return readInternalShorts(buffer, numShorts)
    }

    fun readBytes(): ByteArray? {
        return readInternalBytes()
    }

    private external fun setDeviceId(deviceId: Int)

    private external fun setSampleRate(sampleRate: Int)

    private external fun setBufferSizeInFrames(frames: Int)

    private external fun startRecordingInternal()

    private external fun stopRecordingInternal()

    private external fun readInternalBytes(): ByteArray?

    private external fun readInternalShorts(buffer: ShortArray, numShorts: Int): Int
}