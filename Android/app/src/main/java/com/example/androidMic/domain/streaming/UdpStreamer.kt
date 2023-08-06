package com.example.androidMic.domain.streaming

import android.content.Context
import android.util.Log
import com.example.androidMic.domain.audio.AudioBuffer
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress

private const val TAG: String = "UDP streamer"
class UdpStreamer(val ip: String, val port: Int) : Streamer {

    private val socket: DatagramSocket = DatagramSocket()
    private val address = InetAddress.getByName(ip)
    override fun connect(): Boolean {
        return true
    }

    override fun disconnect(): Boolean {
        return true
    }

    override fun shutdown() {

    }

    override suspend fun stream(audioBuffer: AudioBuffer) {
        if (audioBuffer.isEmpty()) return
        var readSize = 0
        val region = audioBuffer.openReadRegion(Streamer.BUFFER_SIZE)
        val regionSize = region.first
        val regionOffset = region.second
        val packet = DatagramPacket(audioBuffer.buffer, regionOffset, regionSize, address, port)
        try {
            withContext(Dispatchers.IO) {
                socket.send(packet)
                readSize = regionSize
            }
        } catch (e: Exception) {
            Log.d(TAG, "${e.message}")
            readSize = 0
        }
        finally {
            audioBuffer.closeReadRegion(readSize)
        }
    }

    override fun getInfo(): String {
        return "[Device Address]:${ip}"
    }

    override fun isAlive(): Boolean {
        return true
    }
}