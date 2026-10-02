package me.kavishdevar.librepods.presentation.design

import androidx.compose.runtime.compositionLocalOf
import me.kavishdevar.librepods.data.app.AccessibilitySettings

val LocalAccessibilitySettings = compositionLocalOf {
    AccessibilitySettings()
}
