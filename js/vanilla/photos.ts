import L from 'leaflet';
import 'leaflet.markercluster'


// PHOTOS

interface Photo extends L.LatLngLiteral {
  thumbnail: string;
  photoUrl: string;
  caption: string;
  date: string;
}

L.Photo = L.FeatureGroup.extend({
  options: {
    icon: {
      iconSize: [40, 40] as L.PointTuple
    }
  },

  initialize: function (photos: Photo[], options: any) {
    L.setOptions(this, options);
    // @ts-ignore
    L.FeatureGroup.prototype.initialize.call(this, photos);
  },

  addLayers: function (photos: Photo[]) {
    for (const photo of photos)
      this.addLayer(photo);
    return this;
  },

  addLayer: function (photo: Photo) {
    L.FeatureGroup.prototype.addLayer.call(this, this.createMarker(photo));
  },

  createMarker: function (photo: Photo) {
    const marker = L.marker(photo, {
      icon: L.divIcon(L.extend({
        html: `<div style="background-image: url(${photo.thumbnail});"></div>`,
        className: 'leaflet-marker-photo'
      }, photo, this.options.icon)),
      title: photo.caption || ''
    });
    // @ts-ignore
    marker.photo = photo;
    return marker;
  }
});

L.photo = function (photos, options) {
  return new L.Photo(photos, options);
};


if (L.MarkerClusterGroup) {
  L.Photo.Cluster = L.MarkerClusterGroup.extend({
    options: {
      featureGroup: L.photo,
      maxClusterRadius: 100,
      showCoverageOnHover: false,
      icon: { iconSize: [40, 40] as L.PointTuple },

      iconCreateFunction: function(cluster: any) {
        const markers = cluster.getAllChildMarkers();
        return new L.DivIcon(L.extend({
          html: `<div style="background-image: url(${markers[0].photo.thumbnail});"></div><b>${markers.length}</b>`,
          className: 'leaflet-marker-photo',
        }, this.icon));
      },
    },

    initialize: function (options: any) {
      options = L.Util.setOptions(this, options);
      // @ts-ignore
      L.MarkerClusterGroup.prototype.initialize.call(this);
      this._photos = options.featureGroup(null, options);
    },

    add: function (photos: Photo) {
      this.addLayer(this._photos.addLayers(photos));
      return this;
    },

    clear: function () {
      this._photos.clearLayers();
      this.clearLayers();
    }

  });

  L.photo.cluster = function (options: any) {
    return new L.Photo.Cluster!(options);
  };
}


// MAP

const template = `
  <div class="popup">
    <a href="{photo}">
      <img width="{width}" height="{height}" src="{photo}" alt="" />
      <div class="meta">
        <span class="date">{date}</span><span class="caption">{caption}</span>
      </div>
    </a>
  </div>
`;

const map = L.map('map').setView([51.85, 16.57], 13);

L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
  attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
}).addTo(map);

const photoLayer = L.photo.cluster().on('click', function(evt) {
  evt.layer.bindPopup(L.Util.template(template, evt.layer.photo)).openPopup();
});

async function loadData() {
  const data = await fetch('/static/map/data.json');
  if (!data.ok) return;

  return await data.json()
}

// Add photos to the map
loadData().then(data => photoLayer.add(data).addTo(map));
