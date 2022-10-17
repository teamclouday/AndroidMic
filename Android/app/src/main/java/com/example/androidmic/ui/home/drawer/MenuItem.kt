package com.example.androidmic.ui.home.drawer

import androidx.compose.ui.graphics.painter.Painter

data class MenuItem(
    val id: Int,
    val title: String,
    val contentDescription: String,
    val icon: Painter
)
