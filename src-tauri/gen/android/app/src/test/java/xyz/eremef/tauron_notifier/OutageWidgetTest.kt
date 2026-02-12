package xyz.eremef.tauron_notifier

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(manifest = Config.NONE)
class OutageWidgetTest {

    private val provider = OutageWidgetProvider()

    @Test
    fun testParseSettings() {
        val json = """
            {
                "cityGAID": 123,
                "streetGAID": 456,
                "houseNo": "10A",
                "streetName": "Rozbrat",
                "theme": "dark"
            }
        """.trimIndent()

        val settings = provider.parseSettings(json)
        assertEquals(123L, settings?.cityGAID)
        assertEquals(456L, settings?.streetGAID)
        assertEquals("10A", settings?.houseNo)
        assertEquals("Rozbrat", settings?.streetName)
        assertEquals("dark", settings?.theme)
    }

    @Test
    fun testParseSettingsCorrupt() {
        val json = "{ invalid json }"
        val settings = provider.parseSettings(json)
        assertNull(settings)
    }

    @Test
    fun testParseOutageItems() {
        val json = """
            {
                "OutageItems": [
                    { "Message": "Outage at Rozbrat 12, Wrocław" },
                    { "Message": "Maintenance at Legnicka 5, Wrocław" },
                    { "Message": "Broken pipe at Rozbrat 1, Wrocław" }
                ]
            }
        """.trimIndent()

        val count = provider.parseOutageItems(json, "Rozbrat")
        assertEquals(2, count)
    }

    @Test
    fun testParseOutageItemsNoMatch() {
        val json = """
            {
                "OutageItems": [
                    { "Message": "Outage at Main St, Wrocław" }
                ]
            }
        """.trimIndent()

        val count = provider.parseOutageItems(json, "Rozbrat")
        assertEquals(0, count)
    }
}
