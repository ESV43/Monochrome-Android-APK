// js/music-api.js

import { LosslessAPI } from './api.js';
import { YouTubeMusicAPI } from './ytm-api.js';
import { PodcastsAPI } from './podcasts-api.js';
import { musicProviderSettings, ytmSettings } from './storage.js';

/**
 * MusicAPI - Singleton class that provides a unified interface for accessing music streaming services.
 *
 * Supports multiple providers (primarily Tidal and YouTube Music) and includes functionality for searching,
 * retrieving metadata, streaming, and managing playlists, artists, albums, tracks, and podcasts.
 *
 * @class MusicAPI
 * @classdesc Manages API interactions with music providers and provides caching mechanisms
 * for cover artwork and video metadata.
 */
export class MusicAPI {
    static #instance = null;
    /**
     * @type {MusicAPI}
     */
    static get instance() {
        if (!MusicAPI.#instance) {
            throw new Error('MusicAPI not initialized. Call MusicAPI.initialize(settings) first.');
        }
        return MusicAPI.#instance;
    }

    /** @private */
    constructor(settings) {
        this.tidalAPI = new LosslessAPI(settings);
        this.ytmAPI = new YouTubeMusicAPI(ytmSettings);
        this.podcastsAPI = new PodcastsAPI();
        this._settings = settings;
        this.videoArtworkCache = new Map();
    }

    static async initialize(settings) {
        if (MusicAPI.#instance) {
            throw new Error('MusicAPI is already initialized');
        }

        const api = new MusicAPI(settings);
        return (MusicAPI.#instance = api);
    }

    getCurrentProvider() {
        return musicProviderSettings.getProvider();
    }

    // Get the appropriate API based on provider
    getAPI(provider = null) {
        const activeProvider = provider || this.getCurrentProvider();
        if (activeProvider === 'ytm') return this.ytmAPI;
        return this.tidalAPI;
    }

    // Search methods
    async search(query, options = {}) {
        const api = this.getAPI(options.provider);
        if (typeof api.search === 'function') {
            return api.search(query, options);
        }

        // Fallback for providers that don't implement unified search
        const [tracksResult, videosResult, artistsResult, albumsResult, playlistsResult] = await Promise.all([
            api.searchTracks(query, options),
            api.searchVideos ? api.searchVideos(query, options) : Promise.resolve({ items: [] }),
            api.searchArtists(query, options),
            api.searchAlbums(query, options),
            api.searchPlaylists ? api.searchPlaylists(query, options) : Promise.resolve({ items: [] }),
        ]);

        return {
            tracks: tracksResult,
            videos: videosResult,
            artists: artistsResult,
            albums: albumsResult,
            playlists: playlistsResult,
        };
    }

    async searchTracks(query, options = {}) {
        return this.getAPI(options.provider).searchTracks(query, options);
    }

    async searchArtists(query, options = {}) {
        return this.getAPI(options.provider).searchArtists(query, options);
    }

    async searchAlbums(query, options = {}) {
        return this.getAPI(options.provider).searchAlbums(query, options);
    }

    async searchPlaylists(query, options = {}) {
        return this.getAPI(options.provider).searchPlaylists(query, options);
    }

    async searchVideos(query, options = {}) {
        return this.getAPI(options.provider).searchVideos(query, options);
    }

    async searchPodcasts(query, options = {}) {
        return this.podcastsAPI.searchPodcasts(query, options);
    }

    async getPodcast(id, options = {}) {
        return this.podcastsAPI.getPodcastById(id, options);
    }

    async getPodcastEpisodes(id, options = {}) {
        return this.podcastsAPI.getPodcastEpisodes(id, options);
    }

    async getTrendingPodcasts(options = {}) {
        return this.podcastsAPI.getTrendingPodcasts(options);
    }

    // Get methods
    async getTrack(id, quality, provider = null) {
        const api = this.getAPI(provider || this.getProviderFromId(id));
        const cleanId = this.stripProviderPrefix(id);
        return api.getTrack(cleanId, quality);
    }

    async getTrackMetadata(id, provider = null) {
        const api = this.getAPI(provider || this.getProviderFromId(id));
        const cleanId = this.stripProviderPrefix(id);
        return api.getTrackMetadata(cleanId);
    }

    async getAlbum(id, provider = null) {
        const api = this.getAPI(provider || this.getProviderFromId(id));
        const cleanId = this.stripProviderPrefix(id);
        return api.getAlbum(cleanId);
    }

    async getArtist(id, provider = null) {
        const api = this.getAPI(provider || this.getProviderFromId(id));
        const cleanId = this.stripProviderPrefix(id);
        return api.getArtist(cleanId);
    }

    async getArtistBiography(id, provider = null) {
        const api = this.getAPI(provider || this.getProviderFromId(id));
        const cleanId = this.stripProviderPrefix(id);
        if (typeof api.getArtistBiography === 'function') {
            return api.getArtistBiography(cleanId);
        }
        return null;
    }

    async getVideo(id, provider = null) {
        const api = this.getAPI(provider || this.getProviderFromId(id));
        const cleanId = this.stripProviderPrefix(id);
        return api.getVideo(cleanId);
    }

    async getVideoStreamUrl(id, provider = null) {
        const api = this.getAPI(provider || this.getProviderFromId(id));
        const cleanId = this.stripProviderPrefix(id);
        if (typeof api.getVideoStreamUrl === 'function') {
            return api.getVideoStreamUrl(cleanId);
        }
    }

    async getArtistSocials(artistName) {
        return this.tidalAPI.getArtistSocials(artistName);
    }

    async getPlaylist(id, provider = null) {
        const api = this.getAPI(provider || this.getProviderFromId(id));
        return api.getPlaylist(this.stripProviderPrefix(id));
    }

    async getMix(id, provider = null) {
        const api = this.getAPI(provider || this.getProviderFromId(id));
        return api.getMix(this.stripProviderPrefix(id));
    }

    async getTrackRecommendations(id, provider = null) {
        const api = this.getAPI(provider || this.getProviderFromId(id));
        const cleanId = this.stripProviderPrefix(id);
        if (typeof api.getTrackRecommendations === 'function') {
            return api.getTrackRecommendations(cleanId);
        }
        return [];
    }

    // Stream methods
    async getStreamUrl(id, quality, provider = null) {
        const api = this.getAPI(provider || this.getProviderFromId(id));
        const cleanId = this.stripProviderPrefix(id);
        return api.getStreamUrl(cleanId, quality);
    }

    // Cover/artwork methods
    getCoverUrl(id, size = '320', provider = null) {
        if (typeof id === 'string' && id.startsWith('blob:')) {
            return id;
        }
        const api = this.getAPI(provider || this.getProviderFromId(id));
        return api.getCoverUrl(this.stripProviderPrefix(id), size);
    }

    getCoverSrcset(id, provider = null) {
        if (typeof id === 'string' && id.startsWith('blob:')) {
            return '';
        }
        const api = this.getAPI(provider || this.getProviderFromId(id));
        return api.getCoverSrcset(this.stripProviderPrefix(id));
    }

    getVideoCoverUrl(imageId, size = '1280', provider = null) {
        if (!imageId) {
            return null;
        }
        if (typeof imageId === 'string' && imageId.startsWith('blob:')) {
            return imageId;
        }
        const api = this.getAPI(provider || this.getProviderFromId(imageId));
        return api.getVideoCoverUrl(this.stripProviderPrefix(imageId), size);
    }

    async getVideoArtwork(title, artist) {
        const cacheKey = `${title}-${artist}`.toLowerCase();
        if (this.videoArtworkCache.has(cacheKey)) {
            return this.videoArtworkCache.get(cacheKey);
        }
        return null;
    }

    getArtistPictureUrl(id, size = '320', provider = null) {
        const api = this.getAPI(provider || this.getProviderFromId(id));
        return api.getArtistPictureUrl(this.stripProviderPrefix(id), size);
    }

    getArtistPictureSrcset(id, provider = null) {
        const api = this.getAPI(provider || this.getProviderFromId(id));
        return api.getArtistPictureSrcset(this.stripProviderPrefix(id));
    }

    extractStreamUrlFromManifest(manifest, provider = null) {
        const api = this.getAPI(provider);
        return api.extractStreamUrlFromManifest(manifest);
    }

    // Helper methods
    getProviderFromId(id) {
        if (typeof id === 'string') {
            if (id.startsWith('t:')) return 'tidal';
            if (id.startsWith('y:')) return 'ytm';
        }
        return null;
    }

    stripProviderPrefix(id) {
        if (typeof id === 'string') {
            if (id.startsWith('q:') || id.startsWith('t:') || id.startsWith('y:')) {
                return id.slice(2);
            }
        }
        return id;
    }

    // Download methods
    async downloadTrack(id, quality, filename, options = {}) {
        const provider = options.provider || this.getProviderFromId(id);
        const api = this.getAPI(provider);
        const cleanId = this.stripProviderPrefix(id);
        return api.downloadTrack(cleanId, quality, filename, options);
    }

    // Similar/recommendation methods
    async getSimilarArtists(artistId, provider = null) {
        const api = this.getAPI(provider || this.getProviderFromId(artistId));
        const cleanId = this.stripProviderPrefix(artistId);
        return api.getSimilarArtists(cleanId);
    }

    async getArtistTopTracks(artistId, options = {}) {
        const api = this.getAPI(options.provider || this.getProviderFromId(artistId));
        return api.getArtistTopTracks(this.stripProviderPrefix(artistId), options);
    }

    async getSimilarAlbums(albumId, provider = null) {
        const api = this.getAPI(provider || this.getProviderFromId(albumId));
        const cleanId = this.stripProviderPrefix(albumId);
        return api.getSimilarAlbums(cleanId);
    }

    async getRecommendedTracksForPlaylist(tracks, limit = 20, options = {}) {
        const api = this.getAPI(options.provider);
        return api.getRecommendedTracksForPlaylist(tracks, limit, options);
    }

    // Cache methods
    async clearCache() {
        await this.tidalAPI.clearCache();
        await this.ytmAPI.clearCache();
    }

    getCacheStats() {
        return {
            tidal: this.tidalAPI.getCacheStats(),
            ytm: this.ytmAPI.getCacheStats(),
        };
    }

    // Settings accessor for compatibility
    get settings() {
        return this._settings;
    }
}

export const musicAPI = new MusicAPI();
