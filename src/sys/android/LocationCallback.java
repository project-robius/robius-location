/* This file is compiled by build.rs. */

package robius.location;

import android.location.Location;
import android.location.LocationListener;
import java.util.function.Consumer;

// `Consumer<Location>` is implemented for `getCurrentLocation`
// `LocationListener` is implemented for `requestLocationUpdates`

public class LocationCallback implements Consumer<Location>, LocationListener {
  private long weakPtrHigh;
  private long weakPtrLow;

  private native void rustCallback(long weakPtrHigh, long weakPtrLow, Location location);

  public LocationCallback(long weakPtrHigh, long weakPtrLow) {
    this.weakPtrHigh = weakPtrHigh;
    this.weakPtrLow = weakPtrLow;
  }

  public void accept(Location location) {
    rustCallback(this.weakPtrHigh, this.weakPtrLow, location);
  }

  @Override
  public void onLocationChanged(Location location) {
    rustCallback(this.weakPtrHigh, this.weakPtrLow, location);
  }
}
