package io.github.teamclouday.androidMic.domain.streaming

import Message.Messages
import android.content.Context
import android.net.ConnectivityManager
import android.os.Messenger
import android.util.Log
import com.google.protobuf.ByteString
import io.github.teamclouday.androidMic.domain.service.AudioPacket
import io.github.teamclouday.androidMic.domain.service.Command
import io.github.teamclouday.androidMic.domain.service.CommandData
import io.github.teamclouday.androidMic.utils.toBigEndianU32
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import java.io.IOException
import java.net.InetSocketAddress
import java.net.Socket
import java.net.SocketTimeoutException

class TcpStreamer(
    private val scope: CoroutineScope,
    private val tag: String,
    private val ip: String,
    private var port: Int?
) : Streamer {

    private var socket: Socket? = null
    private var streamJob: Job? = null

    companion object {

        fun wifi(
            ctx: Context,
            scope: CoroutineScope,
            ip: String,
            port: Int?
        ): TcpStreamer {

            // check WIFI
            // reference: https://stackoverflow.com/questions/70107145/connectivity-manager-allnetworks-deprecated
            val cm = ctx.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
            val net = cm.activeNetwork
            require(net != null) {
                "Wifi not available"
            }
            require(cm.getNetworkCapabilities(net) != null) {
                "Wifi not available"
            }

            return TcpStreamer(
                scope = scope,
                tag = "WifiStreamer",
                ip = ip,
                port = port
            )
        }

        fun adb(
            scope: CoroutineScope,
        ) = TcpStreamer(
            scope = scope,
            tag = "AdbStreamer",
            ip = "127.0.0.1",
            port = 55555
        )
    }

    // connect to server
    override fun connect(): Boolean {

        val p = port
        if (p != null) {
            val socket = createSocket(p, 1500) ?: return false

            if (!handShake(socket)) {
                Log.d(tag, "connect [Socket]: handshake error")
                socket.close()
                return false
            }
            this.socket = socket
            return true
        } else {
            for (p in DEFAULT_PORT..MAX_PORT) {
                val socket = createSocket(p, 100) ?: continue
                if (!handShake(socket)) {
                    socket.close()
                    continue
                }
                socket.soTimeout = 1500

                this.socket = socket
                this.port = p
                return true
            }
        }

        return false
    }

    // stream data through socket
    override fun start(audioStream: Flow<AudioPacket>, tx: Messenger) {
        streamJob?.cancel()

        streamJob = scope.launch {
            audioStream.collect { data ->
                if (socket == null || socket?.isConnected != true) return@collect

                try {
                    val message = Messages.AudioPacketMessage.newBuilder()
                        .setBuffer(ByteString.copyFrom(data.buffer))
                        .setSampleRate(data.sampleRate)
                        .setAudioFormat(data.audioFormat)
                        .setChannelCount(data.channelCount)
                        .build()
                    val pack = message.toByteArray()

                    socket!!.outputStream.write(pack.size.toBigEndianU32())
                    socket!!.outputStream.write(pack)
                    socket!!.outputStream.flush()
                } catch (e: IOException) {
                    Log.d(tag, "${e.message}")
                    delay(5)
                    disconnect()
                    tx.send(CommandData(Command.StopStream).toCommandMsg())
                } catch (e: Exception) {
                    Log.d(tag, "${e.message}")
                }
            }
        }
    }

    // disconnect from server
    override fun disconnect(): Boolean {
        if (socket == null) return false
        try {
            socket?.close()
        } catch (e: IOException) {
            Log.d(tag, "disconnect [close]: ${e.message}")
            socket = null
            return false
        }
        socket = null
        streamJob?.cancel()
        streamJob = null
        Log.d(tag, "disconnect: complete")
        return true
    }

    // shutdown streamer
    override fun shutdown() {
        disconnect()
    }

    // get connected server information
    override fun getInfo(): String {
        if (socket == null) return ""
        return "[Device Address]:${socket?.remoteSocketAddress}"
    }

    // return true if is connected for streaming
    override fun isAlive(): Boolean {
        return (socket != null && socket?.isConnected == true)
    }


    fun createSocket(p: Int, timeout: Int): Socket? {
        val socket = Socket()
        return try {
            socket.connect(InetSocketAddress(ip, p), timeout)
            socket.soTimeout = timeout
            socket
        } catch (e: IOException) {
            Log.d(tag, "connect [Socket]: ${e.message}")
            null
        } catch (e: SocketTimeoutException) {
            Log.d(tag, "connect [Socket]: ${e.message}")
            null
        }
    }

    fun handShake(
        socket: Socket,
    ): Boolean {

        return try {
            val out = socket.getOutputStream()
            out.write(CHECK_1.toByteArray())
            out.flush()

            val input = socket.getInputStream()
            val msgBuf = ByteArray(CHECK_2.length)
            input.read(msgBuf)
            msgBuf.contentEquals(CHECK_2.toByteArray())

        } catch (_: Exception) {
            false
        }
    }
}