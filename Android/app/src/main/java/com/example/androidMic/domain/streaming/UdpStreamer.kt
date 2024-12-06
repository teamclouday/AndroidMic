package com.example.androidMic.domain.streaming

import android.util.Log
import com.example.androidMic.domain.service.AudioPacket
import com.example.androidMic.utils.toBigEndianU32
import com.google.protobuf.ByteString
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress

class UdpStreamer(private val scope: CoroutineScope, val ip: String, val port: Int) : Streamer {

    private val TAG: String = "UDP streamer"

    private val socket: DatagramSocket = DatagramSocket()
    private val address = InetAddress.getByName(ip)
    private var streamJob: Job? = null

    override fun connect(): Boolean {
        return true
    }

    override fun disconnect(): Boolean {
        streamJob?.cancel()
        streamJob = null
        Log.d(TAG, "disconnect: complete")
        return true
    }

    override fun shutdown() {

    }

    override fun start(audioStream: Flow<AudioPacket>) {
        streamJob?.cancel()

        streamJob = scope.launch {
            audioStream.collect { data ->
                try {
                    val message = Message.AudioPacketMessage.newBuilder()
                        .setBuffer(ByteString.copyFrom(data.buffer))
                        .setSampleRate(data.sampleRate)
                        .setAudioFormat(data.audioFormat)
                        .setChannelCount(data.channelCount)
                        .build()
                    val pack = message.toByteArray()
                    val combined = pack.size.toBigEndianU32() + pack

                    val packet = DatagramPacket(
                        combined,
                        0,
                        combined.size,
                        address,
                        port
                    )

                    socket.send(packet)
                } catch (e: Exception) {
                    Log.d(TAG, "stream: ${e.message}")
                }
            }
        }
    }

    override fun getInfo(): String {
        return "[Device Address]:${ip}"
    }

    override fun isAlive(): Boolean {
        return true
    }
}