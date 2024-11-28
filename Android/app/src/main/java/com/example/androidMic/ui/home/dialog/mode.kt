package com.example.androidMic.ui.home.dialog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import com.example.androidMic.Modes
import com.example.androidMic.ui.MainViewModel

@Composable
fun DialogMode(
    vm: MainViewModel,
    expanded: MutableState<Boolean>,
) {

    ManagerDialog(
        expanded
    ) {
        DialogList(
            enum = Modes.entries,
            onClick = { vm.setMode(it) },
            text = { it.toString() }
        )
    }
}