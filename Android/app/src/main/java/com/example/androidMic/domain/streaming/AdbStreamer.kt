package com.example.androidMic.domain.streaming

import android.content.Context
import android.util.Log
import com.example.androidMic.domain.audio.AudioBuffer
import com.example.androidMic.utils.ignore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import java.io.IOException
import java.lang.IllegalArgumentException
import java.net.ServerSocket
import java.net.Socket

class AdbStreamer(private val ctx: Context,
                  private val port: Int?) : Streamer {

    private val TAG: String = "UsbAdbStreamer"

    private val MAX_WAIT_TIME = 1500 // timeout


    private var mServer : ServerSocket
    private var mSocket : Socket? = null

    init
    {
        try {
            mServer = ServerSocket(port!!)
            mServer.soTimeout = MAX_WAIT_TIME
        } catch (e : Exception) {
            Log.d(TAG, "init failed: ${e.message}")
            throw IllegalArgumentException("USB tcp is not initialized successfully")
        }
    }

    override fun connect(): Boolean
    {
        mSocket = try {
            mServer.accept()
        } catch (e : java.lang.Exception) {
            Log.d(TAG, "accept failed: ${e.message}")
            null
        } ?: return false
        mSocket?.keepAlive = true
        return true
    }

    override fun disconnect(): Boolean
    {
        if(mSocket == null) return false
        try {
            mSocket?.close()
        } catch(e : IOException) {
            Log.d(TAG, "disconnect [close]: ${e.message}")
            mSocket = null
            return false
        }
        mSocket = null
        Log.d(TAG, "disconnect: complete")
        return true
    }

    override fun shutdown()
    {
        disconnect()
        ignore { mServer.close() }
    }

    override suspend fun stream(audioBuffer: AudioBuffer) = withContext(Dispatchers.IO)
    {
        if (mSocket == null || mSocket?.isConnected != true || audioBuffer.isEmpty()) return@withContext
        var readSize = 0

        try {
            val streamOut = mSocket!!.outputStream
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

    override fun getInfo(): String
    {
        if(mSocket == null || mSocket?.isConnected != true) return ""
        return "[USB Mode]\n[Device Address] ${mSocket?.remoteSocketAddress}"
    }

    override fun isAlive(): Boolean
    {
        return (mSocket != null && mSocket?.isConnected == true)
    }

}