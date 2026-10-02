package me.kavishdevar.librepods.presentation.design.typography

import androidx.compose.material3.Typography
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import me.kavishdevar.librepods.R

private val robotoFlex = FontFamily(Font(R.font.roboto_flex))

val RobotoFlexTypography = Typography().run {
    copy(
        displayLarge = displayLarge.copy(fontFamily = robotoFlex),
        displayMedium = displayMedium.copy(fontFamily = robotoFlex),
        displaySmall = displaySmall.copy(fontFamily = robotoFlex),

        headlineLarge = headlineLarge.copy(fontFamily = robotoFlex),
        headlineMedium = headlineMedium.copy(fontFamily = robotoFlex),
        headlineSmall = headlineSmall.copy(fontFamily = robotoFlex),

        titleLarge = titleLarge.copy(fontFamily = robotoFlex),
        titleMedium = titleMedium.copy(fontFamily = robotoFlex),
        titleSmall = titleSmall.copy(fontFamily = robotoFlex),

        bodyLarge = bodyLarge.copy(fontFamily = robotoFlex),
        bodyMedium = bodyMedium.copy(fontFamily = robotoFlex),
        bodySmall = bodySmall.copy(fontFamily = robotoFlex),

        labelLarge = labelLarge.copy(fontFamily = robotoFlex),
        labelMedium = labelMedium.copy(fontFamily = robotoFlex),
        labelSmall = labelSmall.copy(fontFamily = robotoFlex),

        displayLargeEmphasized = displayLargeEmphasized.copy(fontFamily = robotoFlex),
        displayMediumEmphasized = displayMediumEmphasized.copy(fontFamily = robotoFlex),
        displaySmallEmphasized = displaySmallEmphasized.copy(fontFamily = robotoFlex),

        headlineLargeEmphasized = headlineLargeEmphasized.copy(fontFamily = robotoFlex),
        headlineMediumEmphasized = headlineMediumEmphasized.copy(fontFamily = robotoFlex),
        headlineSmallEmphasized = headlineSmallEmphasized.copy(fontFamily = robotoFlex),

        titleLargeEmphasized = titleLargeEmphasized.copy(fontFamily = robotoFlex),
        titleMediumEmphasized = titleMediumEmphasized.copy(fontFamily = robotoFlex),
        titleSmallEmphasized = titleSmallEmphasized.copy(fontFamily = robotoFlex),

        bodyLargeEmphasized = bodyLargeEmphasized.copy(fontFamily = robotoFlex),
        bodyMediumEmphasized = bodyMediumEmphasized.copy(fontFamily = robotoFlex),
        bodySmallEmphasized = bodySmallEmphasized.copy(fontFamily = robotoFlex),

        labelLargeEmphasized = labelLargeEmphasized.copy(fontFamily = robotoFlex),
        labelMediumEmphasized = labelMediumEmphasized.copy(fontFamily = robotoFlex),
        labelSmallEmphasized = labelSmallEmphasized.copy(fontFamily = robotoFlex)
    )
}
