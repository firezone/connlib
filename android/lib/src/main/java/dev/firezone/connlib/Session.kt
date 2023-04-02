package dev.firezone.connlib
import android.util.Log

public object Session {
    public external fun connect(portalURL: String, token: String): Long
    public external fun disconnect(session: Long): Boolean
    public external fun bumpSockets(session: Long): Boolean
    public external fun disableSomeRoamingForBrokenMobileSemantics(session: Long): Boolean

    public fun updateResources(_session: Long, resources: String): Boolean {
        // TODO: Call into client app to update resources list and routing table
        Log.d("Connlib", "updateResources: $resources")

        return true
    }

    public fun setInterfaceAddresses(_session: Long, addresses: String): Boolean {
        // TODO: // Call into client app to update interface addresses
        Log.d("Connlib", "setInterfaceAddresses: $addresses")

        return true
    }
}
