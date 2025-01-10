package com.example.androidMic.ui.home.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import com.example.androidMic.ui.components.ManagerButton

@Composable
fun BaseDialog(
    expanded: MutableState<Boolean>,
    onDismissRequest: (() -> Unit)? = null,
    content: @Composable () -> Unit
) {
    if (expanded.value) {
        Dialog(
            onDismissRequest = {
                onDismissRequest?.invoke()
                expanded.value = false
            }
        ) {
            Surface(
                modifier = Modifier
                    .fillMaxWidth(0.9f)
                    .verticalScroll(rememberScrollState()),
                shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface,
                contentColor = MaterialTheme.colorScheme.onSurface
            ) {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {

                    Spacer(modifier = Modifier.height(25.dp))
                    content()
                    Spacer(modifier = Modifier.height(25.dp))
                }
            }
        }
    }
}

@Composable
fun DialogSpacer() {
    Spacer(modifier = Modifier.height(20.dp))
}

@Composable
fun DialogDivider() {
    HorizontalDivider(
        modifier = Modifier.padding(20.dp),
        color = MaterialTheme.colorScheme.onSurface
    )
}


@Composable
fun <E> DialogList(
    expanded: MutableState<Boolean>,
    enum: List<E>,
    onClick: (E) -> Unit,
    text: (E) -> String,
    bottomContent: (@Composable () -> Unit)? = null
) {

    BaseDialog(
        expanded
    ) {
        enum.forEachIndexed { index, item ->
            ManagerButton(
                onClick = {
                    onClick(item)
                    expanded.value = false
                },
                text = text(item),
                modifier = Modifier.fillMaxWidth(0.8f)
            )

            if (index != enum.indices.last) {
                DialogSpacer()
            }
        }
        bottomContent?.invoke()
    }

}