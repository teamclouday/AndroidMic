package com.example.androidMic.utils

import java.lang.NumberFormatException
import java.net.InetSocketAddress

// helper function to ignore some exceptions
inline fun ignore(body: () -> Unit) {
    try {
        body()
    } catch (e: Exception) {
        e.printStackTrace()
    }
}


fun checkIp(ip: String): Boolean {
    return try {
        InetSocketAddress(ip, 6000)
        true
    } catch (e: Exception) {
        false
    }
}

fun checkPort(portStr: String): Boolean {
    val port = try {
        portStr.toInt()
    } catch (e: NumberFormatException) {
        return false
    }
    return try {
        InetSocketAddress("127.0.0.1", port)
        true
    } catch (e: Exception) {
        false
    }
}