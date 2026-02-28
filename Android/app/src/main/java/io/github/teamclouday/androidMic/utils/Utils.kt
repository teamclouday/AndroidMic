package io.github.teamclouday.androidMic.utils

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
    } catch (_: Exception) {
        false
    }
}

fun checkPort(portStr: String): Boolean {

    if (portStr.isEmpty()) {
        return false
    }

    val port = try {
        portStr.toInt()
    } catch (_: NumberFormatException) {
        return false
    }
    return try {
        InetSocketAddress("127.0.0.1", port)
        true
    } catch (_: Exception) {
        false
    }
}

fun Int.toBigEndianU32(): ByteArray {
    val unsigned = this.toLong() and 0xFFFFFFFFL

    val bytes = ByteArray(4)
    for (i in 0 until 4) {
        bytes[i] = (unsigned shr (24 - i * 8) and 0xFF).toByte()
    }

    return bytes
}

fun ByteArray.chunked(size: Int): List<ByteArray> {
    if (size <= 0) throw IllegalArgumentException("Size must be greater than 0")
    return (0 until size step size).map { start ->
        copyOfRange(
            start, (start + size).coerceAtMost(
                size
            )
        )
    }
}

sealed class Either<out A, out B> {
    class Left<A>(val value: A) : Either<A, Nothing>()
    class Right<B>(val value: B) : Either<Nothing, B>()
}