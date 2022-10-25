package com.example.androidMic.utils

// helper function to ignore some exceptions
inline fun ignore(body: () -> Unit) {
    try {
        body()
    } catch (e: Exception) {
        e.printStackTrace()
    }
}