package io.github.teamclouday.androidMic.domain.streaming

import Message.Messages
import android.os.Messenger
import android.util.Log
import com.google.protobuf.ByteString
import io.github.teamclouday.androidMic.domain.service.AudioPacket
import io.github.teamclouday.androidMic.utils.toBigEndianU32
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress

private const val TAG: String = "UDP streamer"

class UdpStreamer(private val scope: CoroutineScope, val ip: String, var port: Int) : Streamer {



    private val socket: DatagramSocket = DatagramSocket()
    private val address = InetAddress.getByName(ip)
    private var streamJob: Job? = null
    private var sequenceIdx = 0

    override fun connect(): Boolean {
        socket.soTimeout = 1500

        val message = Messages.MessageWrapper.newBuilder()
            .setConnect(
                Messages.ConnectMessage.newBuilder()
                    .build()
            )
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

        try {
            socket.send(packet)
        } catch (_: Exception) {
            return false
        }

        val buff = ByteArray(CHECK_2.length)
        val recvPacket = DatagramPacket(buff, buff.size)

        try {
            socket.receive(recvPacket)
        } catch (_: Exception) {
            return false
        }

        return recvPacket.data.contentEquals(CHECK_2.toByteArray())
    }

    override fun disconnect(): Boolean {
        streamJob?.cancel()
        streamJob = null
        Log.d(TAG, "disconnect: complete")
        return true
    }

    override fun shutdown() {
        disconnect()
    }

    override fun start(audioStream: Flow<AudioPacket>, tx: Messenger) {
        streamJob?.cancel()

        streamJob = scope.launch {
            audioStream.collect { data ->
                try {

                    val message = Messages.MessageWrapper.newBuilder()
                        .setAudioPacket(
                            Messages.AudioPacketMessageOrdered.newBuilder()
                                .setSequenceNumber(sequenceIdx++)
                                .setAudioPacket(
                                    Messages.AudioPacketMessage.newBuilder()
                                        .setBuffer(ByteString.copyFrom(data.buffer))
                                        .setSampleRate(data.sampleRate)
                                        .setAudioFormat(data.audioFormat)
                                        .setChannelCount(data.channelCount)
                                )
                                .build()
                        )
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