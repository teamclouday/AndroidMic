package io.github.teamclouday.AndroidMic.domain.streaming

import Message
import android.content.Context
import android.net.ConnectivityManager
import android.os.Messenger
import android.util.Log
import com.google.protobuf.ByteString
import io.github.teamclouday.AndroidMic.domain.service.AudioPacket
import io.github.teamclouday.AndroidMic.domain.service.Command
import io.github.teamclouday.AndroidMic.domain.service.CommandData
import io.github.teamclouday.AndroidMic.utils.toBigEndianU32
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import java.io.IOException
import java.net.InetSocketAddress
import java.net.Socket
import java.net.SocketTimeoutException

class WifiStreamer(
    ctx: Context,
    private val scope: CoroutineScope,
    ip: String,
    port: Int
) : Streamer {
    private val TAG: String = "MicStreamWIFI"

    private val MAX_WAIT_TIME = 1500 // timeout

    private var socket: Socket? = null
    private var address: String
    private val port: Int
    private var streamJob: Job? = null

    init {
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

        val inetSocketAddress = InetSocketAddress(ip, port)
        this.address = inetSocketAddress.hostName
        this.port = inetSocketAddress.port
    }

    // connect to server
    override fun connect(): Boolean {
        // create socket
        socket = Socket()
        try {
            socket?.connect(InetSocketAddress(address, port), MAX_WAIT_TIME)
        } catch (e: IOException) {
            Log.d(TAG, "connect [Socket]: ${e.message}")
            null
        } catch (e: SocketTimeoutException) {
            Log.d(TAG, "connect [Socket]: ${e.message}")
            null
        } ?: return false
        socket?.soTimeout = MAX_WAIT_TIME

        return true
    }

    // stream data through socket
    override fun start(audioStream: Flow<AudioPacket>, tx: Messenger) {
        streamJob?.cancel()

        streamJob = scope.launch {
            audioStream.collect { data ->
                if (socket == null || socket?.isConnected != true) return@collect

                try {
                    val message = Message.AudioPacketMessage.newBuilder()
                        .setBuffer(ByteString.copyFrom(data.buffer))
                        .setSampleRate(data.sampleRate)
                        .setAudioFormat(data.audioFormat)
                        .setChannelCount(data.channelCount)
                        .build()
                    val pack = message.toByteArray()

                    socket!!.outputStream.write(pack.size.toBigEndianU32())
                    socket!!.outputStream.write(message.toByteArray())
                    socket!!.outputStream.flush()
                } catch (e: IOException) {
                    Log.d(TAG, "${e.message}")
                    delay(5)
                    disconnect()
                    tx.send(CommandData(Command.StopStream).toCommandMsg())
                } catch (e: Exception) {
                    Log.d(TAG, "${e.message}")
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
            Log.d(TAG, "disconnect [close]: ${e.message}")
            socket = null
            return false
        }
        socket = null
        streamJob?.cancel()
        streamJob = null
        Log.d(TAG, "disconnect: complete")
        return true
    }

    // shutdown streamer
    override fun shutdown() {
        disconnect()
        address = ""
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
}