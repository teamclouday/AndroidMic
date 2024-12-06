package com.example.androidMic.domain.streaming

import android.util.Log
import com.example.androidMic.domain.service.AudioPacket
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.Flow
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
//        val moshiPack = MoshiPack()
//
//        streamJob = scope.launch {
//            audioStream.collect { data ->
//                try {
//                    val packed = moshiPack.pack(data).readByteArray()
//                    val packet = DatagramPacket(
//                        packed,
//                        0,
//                        packed.size,
//                        address,
//                        port
//                    )
//                    socket.send(packet)
//                } catch (e: Exception) {
//                    Log.d(TAG, "${e.message}")
//                }
//            }
//        }
    }

    override fun getInfo(): String {
        return "[Device Address]:${ip}"
    }

    override fun isAlive(): Boolean {
        return true
    }
}