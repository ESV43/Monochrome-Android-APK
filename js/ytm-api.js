// js/ytm-api.js
import { APICache } from './cache.js';
import { PreparedTrack, PreparedVideo, Album, TrackAlbum } from './container-classes.js';

export class YouTubeMusicAPI {
    constructor(settings) {
        this.settings = settings;
        this.cache = new APICache({
            maxSize: 200,
            ttl: 1000 * 60 * 60, // 1 hour
        });
        this.baseUrl = 'https://music.youtube.com/youtubei/v1';
        this.apiKey = null; // Usually extracted from the page, but InnerTube works with a static one too
        this.clientVersion = '1.20240101.01.00';
    }

    async _fetch(endpoint, body = {}) {
        const credentials = this.settings.getCredentials();
        const headers = {
            'Content-Type': 'application/json',
            'X-Goog-AuthUser': credentials.authUser || '0',
            'X-Origin': 'https://music.youtube.com',
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
        };

        if (credentials.cookie) {
            headers['Cookie'] = credentials.cookie;
        }

        const fullBody = {
            context: {
                client: {
                    clientName: 'WEB_REMIX',
                    clientVersion: this.clientVersion,
                    hl: 'en',
                    gl: 'US',
                },
                user: {
                    lockedSafetyMode: false,
                },
            },
            ...body,
        };

        // We use a proxy if running in browser, or direct in Capacitor if possible
        const url = `${this.baseUrl}/${endpoint}?key=${this.apiKey || ''}`;
        
        const response = await fetch(url, {
            method: 'POST',
            headers,
            body: JSON.stringify(fullBody),
        });

        if (!response.ok) {
            throw new Error(`YTM API Error: ${response.status}`);
        }

        return await response.json();
    }

    async search(query, options = {}) {
        const data = await this._fetch('search', {
            query,
            params: options.params || 'EgWKAQIIAWoKEAkQBRAKEAMQBA%3D%3D', // Example filter for Songs
        });

        return this._parseSearchResponse(data);
    }

    _parseSearchResponse(data) {
        const results = {
            tracks: { items: [] },
            albums: { items: [] },
            artists: { items: [] },
            playlists: { items: [] },
        };

        // InnerTube traversal
        const shelf = data.contents?.tabbedSearchResultsRenderer?.tabs?.[0]?.tabRenderer?.content?.sectionListRenderer?.contents;
        if (!shelf) return results;

        shelf.forEach(section => {
            const items = section.musicShelfRenderer?.contents || section.musicCardShelfRenderer?.contents;
            if (!items) return;

            items.forEach(item => {
                const renderer = item.musicResponsiveListItemRenderer;
                if (!renderer) return;

                const track = this._parseRenderer(renderer);
                if (track) results.tracks.items.push(track);
            });
        });

        return results;
    }

    _parseRenderer(renderer) {
        try {
            const videoId = renderer.playlistItemData?.videoId || 
                          renderer.navigationEndpoint?.watchEndpoint?.videoId;
            if (!videoId) return null;

            const title = renderer.flexColumns[0].musicResponsiveListItemFlexColumnRenderer.text.runs[0].text;
            const artistRun = renderer.flexColumns[1].musicResponsiveListItemFlexColumnRenderer.text.runs;
            const artistName = artistRun[0].text;
            const albumName = artistRun.length > 2 ? artistRun[2].text : null;
            const durationStr = artistRun[artistRun.length - 1].text;

            return new PreparedTrack({
                id: `y:${videoId}`,
                title,
                artist: { name: artistName },
                artists: [{ name: artistName }],
                album: { title: albumName, id: `y:album:${videoId}` },
                duration: this._parseDuration(durationStr),
                audioQuality: 'HIGH',
                provider: 'ytm'
            });
        } catch (e) {
            return null;
        }
    }

    _parseDuration(str) {
        if (!str || !str.includes(':')) return 0;
        const parts = str.split(':').map(Number);
        if (parts.length === 2) return parts[0] * 60 + parts[1];
        if (parts.length === 3) return parts[0] * 3600 + parts[1] * 60 + parts[2];
        return 0;
    }

    async getStreamUrl(id) {
        const videoId = id.startsWith('y:') ? id.slice(2) : id;
        const data = await this._fetch('player', {
            videoId,
            playbackContext: {
                contentPlaybackContext: {
                    signatureTimestamp: 19745, 
                }
            }
        });

        const formats = data.streamingData?.adaptiveFormats || [];
        const audioFormat = formats
            .filter(f => f.mimeType.includes('audio/mp4'))
            .sort((a, b) => (b.bitrate || 0) - (a.bitrate || 0))[0];
        
        return audioFormat?.url || null;
    }

    async getUserPlaylists() {
        const data = await this._fetch('browse', {
            browseId: 'FEmusic_library_list_visual',
        });
        
        const playlists = [];
        const items = data.contents?.singleColumnBrowseResultsRenderer?.tabs?.[0]?.tabRenderer?.content?.sectionListRenderer?.contents?.[0]?.gridRenderer?.items;
        
        if (items) {
            items.forEach(item => {
                const renderer = item.musicTwoColumnItemRenderer || item.musicResponsiveListItemRenderer;
                if (!renderer) return;
                
                const title = renderer.title?.runs?.[0]?.text || renderer.flexColumns?.[0]?.musicResponsiveListItemFlexColumnRenderer?.text?.runs?.[0]?.text;
                const browseId = renderer.navigationEndpoint?.browseEndpoint?.browseId;
                
                if (title && browseId) {
                    playlists.push({
                        id: `y:${browseId}`,
                        title,
                        type: 'playlist',
                        provider: 'ytm'
                    });
                }
            });
        }
        
        return playlists;
    }

    async getPlaylist(id) {
        const browseId = id.startsWith('y:') ? id.slice(2) : id;
        const data = await this._fetch('browse', {
            browseId,
        });

        const title = data.header?.musicDetailHeaderRenderer?.title?.runs?.[0]?.text || 'YouTube Playlist';
        const tracks = [];
        
        const contents = data.contents?.singleColumnBrowseResultsRenderer?.tabs?.[0]?.tabRenderer?.content?.sectionListRenderer?.contents?.[0]?.musicPlaylistShelfRenderer?.contents;
        
        if (contents) {
            contents.forEach(item => {
                const renderer = item.musicResponsiveListItemRenderer;
                if (!renderer) return;
                
                const track = this._parseRenderer(renderer);
                if (track) tracks.push(track);
            });
        }

        return {
            id: `y:${browseId}`,
            title,
            tracks,
            type: 'playlist',
            provider: 'ytm'
        };
    }

    async clearCache() {
        await this.cache.clear();
    }

    getCacheStats() {
        return { size: this.cache.size };
    }
}
