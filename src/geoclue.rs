use async_stream::stream;
use futures::{Stream, StreamExt};
use zbus::{Connection, proxy};
use zvariant::ObjectPath;

#[derive(Debug, Clone, Copy)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

pub async fn location_coordinates_stream()
-> color_eyre::Result<impl Stream<Item = color_eyre::Result<Coordinates>>> {
    let connection = Connection::system().await?;

    let geoclue_manager = ManagerProxy::new(&connection).await?;
    let geoclue_client = geoclue_manager.get_client().await?;

    geoclue_client.set_desktop_id("automatic-redshift").await?;
    geoclue_client.set_distance_threshold(10000).await?; // meters
    geoclue_client
        .set_requested_accuracy_level(AccuracyLevel::City as u32)
        .await?;

    let mut location_updated = geoclue_client.receive_location_updated().await?;
    geoclue_client.start().await?;

    let coordinates_stream = stream! {
        while let Some(signal) = location_updated.next().await {
            let args = signal.args()?;

                let location = LocationProxy::builder(&connection)
                    .path(args.new())?
                    .build()
                    .await?;

                let latitude = location.latitude().await?;
                let longitude = location.longitude().await?;

               yield Ok::<Coordinates, color_eyre::Report>(Coordinates {
                    latitude,
                    longitude,
                });
        }
    };

    Ok(coordinates_stream)
}

#[allow(dead_code)]
enum AccuracyLevel {
    None = 0,
    Country = 1,
    City = 4,
    Neighborhood = 5,
    Street = 6,
    Exact = 8,
}

#[proxy(
    default_service = "org.freedesktop.GeoClue2",
    interface = "org.freedesktop.GeoClue2.Manager",
    default_path = "/org/freedesktop/GeoClue2/Manager"
)]
trait Manager {
    /// GetClient method
    #[zbus(object = "Client")]
    fn get_client(&self);
}

#[proxy(
    default_service = "org.freedesktop.GeoClue2",
    interface = "org.freedesktop.GeoClue2.Client"
)]
trait Client {
    /// Start method
    fn start(&self) -> zbus::Result<()>;

    /// Stop method
    fn stop(&self) -> zbus::Result<()>;

    /// LocationUpdated signal
    #[zbus(signal)]
    fn location_updated(&self, old: ObjectPath<'_>, new: ObjectPath<'_>) -> zbus::Result<()>;

    /// DesktopId property
    #[zbus(property)]
    fn set_desktop_id(&self, id: &str) -> zbus::Result<()>;

    /// DistanceThreshold property
    #[zbus(property)]
    fn set_distance_threshold(&self, meters: u32) -> zbus::Result<()>;

    /// RequestedAccuracyLevel property
    #[zbus(property)]
    fn set_requested_accuracy_level(&self, level: u32) -> zbus::Result<()>;
}

#[proxy(
    default_service = "org.freedesktop.GeoClue2",
    interface = "org.freedesktop.GeoClue2.Location"
)]
trait Location {
    /// Latitude property
    #[zbus(property)]
    fn latitude(&self) -> zbus::Result<f64>;

    /// Longitude property
    #[zbus(property)]
    fn longitude(&self) -> zbus::Result<f64>;
}
