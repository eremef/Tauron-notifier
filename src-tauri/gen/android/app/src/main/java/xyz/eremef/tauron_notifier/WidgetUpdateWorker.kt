package xyz.eremef.tauron_notifier

import android.appwidget.AppWidgetManager
import android.content.ComponentName
import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters

class WidgetUpdateWorker(
    private val context: Context,
    workerParams: WorkerParameters
) : CoroutineWorker(context, workerParams) {

    override suspend fun doWork(): Result {
        val appWidgetManager = AppWidgetManager.getInstance(context)
        val componentName = ComponentName(context, OutageWidgetProvider::class.java)
        val appWidgetIds = appWidgetManager.getAppWidgetIds(componentName)

        val provider = OutageWidgetProvider()
        for (appWidgetId in appWidgetIds) {
            provider.updateWidget(context, appWidgetManager, appWidgetId)
        }

        return Result.success()
    }
}
