package io.github.teamclouday.AndroidMic.domain.streaming

import Message
import android.os.Messenger
import android.util.Log
import com.google.protobuf.ByteString
import io.github.teamclouday.AndroidMic.domain.service.AudioPacket
import io.github.teamclouday.AndroidMic.utils.toBigEndianU32
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress

class UdpStreamer(private val scope: CoroutineScope, val ip: String, var port: Int?) : Streamer {

    private val TAG: String = "UDP streamer"

    private val socket: DatagramSocket = DatagramSocket()
    private val address = InetAddress.getByName(ip)
    private var streamJob: Job? = null
    private var sequenceIdx = 0

    override fun connect(): Boolean {

        val p = port
        if (p != null) {

            if (!handShake(p, 1500)) {
                Log.d(TAG, "connect [Socket]: handshake error")
                socket.close()
                return false
            }
            return true
        } else {
            for (p in DEFAULT_PORT..MAX_PORT) {
                if (!handShake(p, 100)) {
                    continue
                }
                this.port = p
                return true
            }
        }

        return false
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
                    val message = Message.AudioPacketMessageOrdered.newBuilder()
                        .setSequenceNumber(sequenceIdx++)
                        .setAudioPacket(
                            Message.AudioPacketMessage.newBuilder()
                                .setBuffer(ByteString.copyFrom(data.buffer))
                                .setSampleRate(data.sampleRate)
                                .setAudioFormat(data.audioFormat)
                                .setChannelCount(data.channelCount)
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
                        port!!
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

    fun handShake(
        p: Int,
        timeout: Int
    ): Boolean {

        return try {

            val array = CHECK_1.toByteArray()
            val packet = DatagramPacket(
                array,
                0,
                array.size,
                address,
                p
            )

            socket.send(packet)

            val buff = ByteArray(CHECK_2.length)
            val recvPacket = DatagramPacket(buff, buff.size)

            socket.soTimeout = timeout
            socket.receive(recvPacket)

            recvPacket.data.contentEquals(CHECK_2.toByteArray())
        } catch (_: Exception) {
            false
        }
    }
}