/* This file is compiled by build.rs. */

package robius.location;

import android.location.Location;
import java.util.function.Consumer;

public class LocationCallback implements Consumer<Location> {
  private long handlerPtr;
  private long handlerFnPtr;
  private long handlerErrFnPtr;

  private native void rustCallback(long handlerPtr, long handlerFnPtr, long HandlerErrFnPtr, Location location);

  public LocationCallback(long handlerPtr, long handlerFnPtr, long handlerErrFnPtr) {
    this.handlerPtr = handlerPtr;
    this.handlerFnPtr = handlerFnPtr;
    this.handlerErrFnPtr = handlerErrFnPtr;
  }

  public void accept(Location location) {
    rustCallback(this.handlerPtr, this.handlerFnPtr, this.handlerErrFnPtr, location);
  }
}