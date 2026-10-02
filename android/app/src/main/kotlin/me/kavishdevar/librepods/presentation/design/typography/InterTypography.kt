package me.kavishdevar.librepods.presentation.design.typography

import androidx.compose.material3.Typography
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import me.kavishdevar.librepods.R


private val interFamily = FontFamily(Font(R.font.inter))

val InterTypography = Typography().run {
    copy(
        displayLarge = displayLarge.copy(fontFamily = interFamily),
        displayMedium = displayMedium.copy(fontFamily = interFamily),
        displaySmall = displaySmall.copy(fontFamily = interFamily),

        headlineLarge = headlineLarge.copy(fontFamily = interFamily),
        headlineMedium = headlineMedium.copy(fontFamily = interFamily),
        headlineSmall = headlineSmall.copy(fontFamily = interFamily),

        titleLarge = titleLarge.copy(fontFamily = interFamily),
        titleMedium = titleMedium.copy(fontFamily = interFamily),
        titleSmall = titleSmall.copy(fontFamily = interFamily),

        bodyLarge = bodyLarge.copy(fontFamily = interFamily),
        bodyMedium = bodyMedium.copy(fontFamily = interFamily),
        bodySmall = bodySmall.copy(fontFamily = interFamily),

        labelLarge = labelLarge.copy(fontFamily = interFamily),
        labelMedium = labelMedium.copy(fontFamily = interFamily),
        labelSmall = labelSmall.copy(fontFamily = interFamily),

        displayLargeEmphasized = displayLargeEmphasized.copy(fontFamily = interFamily),
        displayMediumEmphasized = displayMediumEmphasized.copy(fontFamily = interFamily),
        displaySmallEmphasized = displaySmallEmphasized.copy(fontFamily = interFamily),

        headlineLargeEmphasized = headlineLargeEmphasized.copy(fontFamily = interFamily),
        headlineMediumEmphasized = headlineMediumEmphasized.copy(fontFamily = interFamily),
        headlineSmallEmphasized = headlineSmallEmphasized.copy(fontFamily = interFamily),

        titleLargeEmphasized = titleLargeEmphasized.copy(fontFamily = interFamily),
        titleMediumEmphasized = titleMediumEmphasized.copy(fontFamily = interFamily),
        titleSmallEmphasized = titleSmallEmphasized.copy(fontFamily = interFamily),

        bodyLargeEmphasized = bodyLargeEmphasized.copy(fontFamily = interFamily),
        bodyMediumEmphasized = bodyMediumEmphasized.copy(fontFamily = interFamily),
        bodySmallEmphasized = bodySmallEmphasized.copy(fontFamily = interFamily),

        labelLargeEmphasized = labelLargeEmphasized.copy(fontFamily = interFamily),
        labelMediumEmphasized = labelMediumEmphasized.copy(fontFamily = interFamily),
        labelSmallEmphasized = labelSmallEmphasized.copy(fontFamily = interFamily)
    )
}
