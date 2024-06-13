/* This file is compiled by build.rs. */

package robius.location;

import android.location.Location;
import java.util.function.Consumer;

public class LocationCallback implements Consumer<Location> {
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
}