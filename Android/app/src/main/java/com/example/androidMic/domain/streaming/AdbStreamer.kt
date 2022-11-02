package com.example.androidMic.domain.streaming

import android.util.Log
import com.example.androidMic.domain.audio.AudioBuffer
import com.example.androidMic.utils.ignore
import kotlinx.coroutines.*
import java.io.DataInputStream
import java.io.DataOutputStream
import java.io.IOException
import java.net.InetSocketAddress
import java.net.Socket
import java.net.SocketTimeoutException

class AdbStreamer(val port: Int) : Streamer {

    private val TAG: String = "UsbAdbStreamer"

    private val MAX_WAIT_TIME = 1500 // timeout

    private var socket: Socket? = null
    private val address: String = "127.0.0.1"

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
        // test server
        if (!testConnection(socket!!)) {
            socket?.close()
            socket = null
            return false
        }
        return true
    }

    // stream data through socket
    override suspend fun stream(audioBuffer: AudioBuffer) = withContext(Dispatchers.IO)
    {
        if (socket == null || socket?.isConnected != true || audioBuffer.isEmpty()) return@withContext
        var readSize = 0
        try {
            val streamOut = socket!!.outputStream
            val region = audioBuffer.openReadRegion(Streamer.BUFFER_SIZE)
            val regionSize = region.first
            val regionOffset = region.second
            streamOut.write(audioBuffer.buffer, regionOffset, regionSize)
            readSize = regionSize
            // streamOut.flush()
        } catch (e: IOException) {
            Log.d(TAG, "${e.message}")
            delay(5)
            disconnect()
            readSize = 0
        } catch (e: Exception) {
            Log.d(TAG, "${e.message}")
            readSize = 0
        } finally {
            audioBuffer.closeReadRegion(readSize)
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
        Log.d(TAG, "disconnect: complete")
        return true
    }

    // shutdown streamer
    override fun shutdown() {
        disconnect()
    }

    // test connection
    private fun testConnection(socket: Socket): Boolean {
        if (!socket.isConnected) return false
        var isValid = false
        runBlocking(Dispatchers.IO) {
            val job = launch {
                ignore {
                    val streamOut = DataOutputStream(socket.outputStream)
                    streamOut.write(Streamer.DEVICE_CHECK.toByteArray(Charsets.UTF_8))
                    streamOut.flush()
                    val streamIn = DataInputStream(socket.inputStream)
                    val buffer = ByteArray(100)
                    val size = streamIn.read(buffer, 0, 100)
                    val received = String(buffer, 0, size, Charsets.UTF_8)
                    if (received == Streamer.DEVICE_CHECK_EXPECT) {
                        isValid = true
                        Log.d(TAG, "testConnection: device matched!")
                    } else
                        Log.d(TAG, "testConnection: device mismatch with $received!")
                }
            }
            var time = 5
            while (!job.isCompleted && time < MAX_WAIT_TIME) {
                delay(5)
                time += 5
            }
            if (!job.isCompleted) {
                job.cancel()
                Log.d(TAG, "testConnection: timeout!")
            }
        }
        return isValid
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