//! Range-based relevance judgments for the media-server codebase
//!
//! Defines ground truth for 45 benchmark queries:
//! - G1: Qualified symbol queries (10) - symbol tokens in natural-language
//!   structure (e.g. `flv_header_read in flv-header`), Strong only
//! - G2: Semantic queries (20) - Strong + Related, single-entity ranges
//! - FZ: Fuzzy queries (15) - lexical perturbations of G1 sources
//!
//! All line ranges verified against fixtures/cpp/review/media-server/.
//! Each range is confined to a single function definition to avoid
//! chunking boundary issues.

use crate::bench_data::{QueryType, RelevanceJudgment, RelevanceLevel, SourceRange};

pub fn media_server_relevance_judgments() -> Vec<RelevanceJudgment> {
    use RelevanceLevel as RL;

    vec![
        // ==================== G1: Qualified Symbol Queries (10 total) ====================

        // G1Q1: flv_header_read - FLV file header parser
        RelevanceJudgment {
            id: "G1Q1".into(),
            query_text: "flv_header_read in flv-header".into(),
            query_type: QueryType::Qualified,
            fuzzy_subtype: None,
            relevant_ranges: vec![(
                SourceRange {
                    file: "libflv/source/flv-header.c".into(),
                    start_line: 25,
                    end_line: 44,
                },
                RL::Strong,
            )],
        },
        // G1Q2: mov_read_stts - MP4 decoding-time-to-sample box reader
        RelevanceJudgment {
            id: "G1Q2".into(),
            query_text: "mov_read_stts in mov-stts".into(),
            query_type: QueryType::Qualified,
            fuzzy_subtype: None,
            relevant_ranges: vec![(
                SourceRange {
                    file: "libmov/source/mov-stts.c".into(),
                    start_line: 7,
                    end_line: 33,
                },
                RL::Strong,
            )],
        },
        // G1Q3: rtp_packet_serialize - RTP packet serializer
        RelevanceJudgment {
            id: "G1Q3".into(),
            query_text: "rtp_packet_serialize in rtp-packet".into(),
            query_type: QueryType::Qualified,
            fuzzy_subtype: None,
            relevant_ranges: vec![(
                SourceRange {
                    file: "librtp/source/rtp-packet.c".into(),
                    start_line: 140,
                    end_line: 150,
                },
                RL::Strong,
            )],
        },
        // G1Q4: rtsp_header_session - RTSP Session header parser
        RelevanceJudgment {
            id: "G1Q4".into(),
            query_text: "rtsp_header_session in rtsp-header-session".into(),
            query_type: QueryType::Qualified,
            fuzzy_subtype: None,
            relevant_ranges: vec![(
                SourceRange {
                    file: "librtsp/source/rtsp-header-session.c".into(),
                    start_line: 11,
                    end_line: 47,
                },
                RL::Strong,
            )],
        },
        // G1Q5: rtmp_chunk_basic_header_read - RTMP chunk basic header decoder
        RelevanceJudgment {
            id: "G1Q5".into(),
            query_text: "rtmp_chunk_basic_header_read in rtmp-chunk-header".into(),
            query_type: QueryType::Qualified,
            fuzzy_subtype: None,
            relevant_ranges: vec![(
                SourceRange {
                    file: "librtmp/source/rtmp-chunk-header.c".into(),
                    start_line: 21,
                    end_line: 40,
                },
                RL::Strong,
            )],
        },
        // G1Q6: hls_playlist_duration - HLS playlist duration summation
        RelevanceJudgment {
            id: "G1Q6".into(),
            query_text: "hls_playlist_duration in hls-playlist".into(),
            query_type: QueryType::Qualified,
            fuzzy_subtype: None,
            relevant_ranges: vec![(
                SourceRange {
                    file: "libhls/source/hls-playlist.c".into(),
                    start_line: 3,
                    end_line: 15,
                },
                RL::Strong,
            )],
        },
        // G1Q7: mpeg_ts_add_stream - MPEG-TS stream registration
        RelevanceJudgment {
            id: "G1Q7".into(),
            query_text: "mpeg_ts_add_stream in mpeg-ts-enc".into(),
            query_type: QueryType::Qualified,
            fuzzy_subtype: None,
            relevant_ranges: vec![(
                SourceRange {
                    file: "libmpeg/source/mpeg-ts-enc.c".into(),
                    start_line: 556,
                    end_line: 571,
                },
                RL::Strong,
            )],
        },
        // G1Q8: sip_message_isinvite - SIP INVITE method check
        RelevanceJudgment {
            id: "G1Q8".into(),
            query_text: "sip_message_isinvite in sip-message".into(),
            query_type: QueryType::Qualified,
            fuzzy_subtype: None,
            relevant_ranges: vec![(
                SourceRange {
                    file: "libsip/src/sip-message.c".into(),
                    start_line: 367,
                    end_line: 370,
                },
                RL::Strong,
            )],
        },
        // G1Q9: sdp_h264_load - SDP sprop-parameter-sets decoder
        RelevanceJudgment {
            id: "G1Q9".into(),
            query_text: "sdp_h264_load in sdp-h264".into(),
            query_type: QueryType::Qualified,
            fuzzy_subtype: None,
            relevant_ranges: vec![(
                SourceRange {
                    file: "librtsp/source/sdp/sdp-h264.c".into(),
                    start_line: 59,
                    end_line: 83,
                },
                RL::Strong,
            )],
        },
        // G1Q10: pat_find - MPEG PAT program lookup
        RelevanceJudgment {
            id: "G1Q10".into(),
            query_text: "pat_find in mpeg-pat".into(),
            query_type: QueryType::Qualified,
            fuzzy_subtype: None,
            relevant_ranges: vec![(
                SourceRange {
                    file: "libmpeg/source/mpeg-pat.c".into(),
                    start_line: 177,
                    end_line: 186,
                },
                RL::Strong,
            )],
        },
        // ==================== G2: Semantic Queries (20 total) ====================

        // G2Q1: parse FLV file and tag headers
        RelevanceJudgment {
            id: "G2Q1".into(),
            query_text: "validate FLV file signature and decode tag headers with timestamp".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "libflv/source/flv-header.c".into(),
                        start_line: 25,
                        end_line: 44,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "libflv/source/flv-header.c".into(),
                        start_line: 46,
                        end_line: 70,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "libflv/source/flv-header.c".into(),
                        start_line: 273,
                        end_line: 288,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q2: MP4 decoding-time table round-trip
        RelevanceJudgment {
            id: "G2Q2".into(),
            query_text: "read MP4 decoding time table and expand sample timestamps".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "libmov/source/mov-stts.c".into(),
                        start_line: 7,
                        end_line: 33,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "libmov/source/mov-stts.c".into(),
                        start_line: 204,
                        end_line: 218,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "libmov/source/mov-stts.c".into(),
                        start_line: 145,
                        end_line: 170,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q3: RTP packet output with jitter buffering
        RelevanceJudgment {
            id: "G2Q3".into(),
            query_text: "serialize RTP packet and reorder arrivals in jitter buffer".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "librtp/source/rtp-packet.c".into(),
                        start_line: 140,
                        end_line: 150,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "librtp/source/rtp-queue.c".into(),
                        start_line: 292,
                        end_line: 322,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q4: H264 over RTP fragmentation choice
        RelevanceJudgment {
            id: "G2Q4".into(),
            query_text: "packetize H264 NAL units choosing single packet or FU-A fragments".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "librtp/payload/rtp-h264-pack.c".into(),
                        start_line: 80,
                        end_line: 104,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "librtp/payload/rtp-h264-pack.c".into(),
                        start_line: 164,
                        end_line: 177,
                    },
                    RL::Strong,
                ),
            ],
        },
        // G2Q5: WebRTC absolute send time extension
        RelevanceJudgment {
            id: "G2Q5".into(),
            query_text: "decode and encode WebRTC absolute send time header extension".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "librtp/rtpext/rtp-ext-abs-send-time.c".into(),
                        start_line: 20,
                        end_line: 28,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "librtp/rtpext/rtp-ext-abs-send-time.c".into(),
                        start_line: 30,
                        end_line: 40,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q6: RTMP chunk header codec
        RelevanceJudgment {
            id: "G2Q6".into(),
            query_text: "decode RTMP chunk headers conditionally by format type".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "librtmp/source/rtmp-chunk-header.c".into(),
                        start_line: 21,
                        end_line: 40,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "librtmp/source/rtmp-chunk-header.c".into(),
                        start_line: 75,
                        end_line: 102,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "librtmp/source/rtmp-chunk-header.c".into(),
                        start_line: 42,
                        end_line: 62,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q7: RTMP handshake messages
        RelevanceJudgment {
            id: "G2Q7".into(),
            query_text: "build RTMP handshake messages with timestamp and random bytes".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "librtmp/source/rtmp-handshake.c".into(),
                        start_line: 200,
                        end_line: 212,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "librtmp/source/rtmp-handshake.c".into(),
                        start_line: 193,
                        end_line: 198,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q8: RTMP user control events
        RelevanceJudgment {
            id: "G2Q8".into(),
            query_text: "send stream-begin notification and dispatch user control events".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "librtmp/source/rtmp-event.c".into(),
                        start_line: 38,
                        end_line: 45,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "librtmp/source/rtmp-event.c".into(),
                        start_line: 108,
                        end_line: 143,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q9: FLV muxing of AVC frames
        RelevanceJudgment {
            id: "G2Q9".into(),
            query_text: "mux H264 Annex-B frames into FLV video tags".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "libflv/source/flv-muxer.c".into(),
                        start_line: 344,
                        end_line: 358,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "libflv/source/flv-header.c".into(),
                        start_line: 290,
                        end_line: 319,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q10: FLV demux dispatch
        RelevanceJudgment {
            id: "G2Q10".into(),
            query_text: "dispatch FLV tags by type to audio video or script handlers".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "libflv/source/flv-demuxer.c".into(),
                        start_line: 336,
                        end_line: 361,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "libflv/source/flv-header.c".into(),
                        start_line: 46,
                        end_line: 70,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q11: HLS playlist type probing
        RelevanceJudgment {
            id: "G2Q11".into(),
            query_text: "sniff HLS manifest to distinguish media playlist from master".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "libhls/source/hls-parser.c".into(),
                        start_line: 775,
                        end_line: 791,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "libhls/source/hls-playlist.c".into(),
                        start_line: 3,
                        end_line: 15,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q12: HLS variant selection
        RelevanceJudgment {
            id: "G2Q12".into(),
            query_text: "select HLS variant with highest bandwidth for playback".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "libhls/source/hls-master.c".into(),
                        start_line: 63,
                        end_line: 78,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "libhls/source/hls-parser.c".into(),
                        start_line: 775,
                        end_line: 791,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q13: MPEG-TS program and stream setup
        RelevanceJudgment {
            id: "G2Q13".into(),
            query_text: "create MPEG-TS program and register elementary streams in PMT".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "libmpeg/source/mpeg-ts-enc.c".into(),
                        start_line: 556,
                        end_line: 571,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "libmpeg/source/mpeg-ts-enc.c".into(),
                        start_line: 573,
                        end_line: 588,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "libmpeg/source/mpeg-pat.c".into(),
                        start_line: 177,
                        end_line: 186,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q14: SIP method checks
        RelevanceJudgment {
            id: "G2Q14".into(),
            query_text: "check whether SIP request uses INVITE or REGISTER method".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "libsip/src/sip-message.c".into(),
                        start_line: 367,
                        end_line: 370,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "libsip/src/sip-message.c".into(),
                        start_line: 372,
                        end_line: 375,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q15: RTSP session negotiation
        RelevanceJudgment {
            id: "G2Q15".into(),
            query_text: "parse RTSP session header and decode SDP video parameters".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "librtsp/source/rtsp-header-session.c".into(),
                        start_line: 11,
                        end_line: 47,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "librtsp/source/sdp/sdp-h264.c".into(),
                        start_line: 59,
                        end_line: 83,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q16: SDP H264 offer round-trip
        RelevanceJudgment {
            id: "G2Q16".into(),
            query_text: "generate SDP video description and parse back parameter sets".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "librtsp/source/sdp/sdp-h264.c".into(),
                        start_line: 59,
                        end_line: 83,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "librtsp/source/sdp/sdp-h264.c".into(),
                        start_line: 13,
                        end_line: 57,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q17: RTP receive path
        RelevanceJudgment {
            id: "G2Q17".into(),
            query_text: "serialize RTP fixed header and handle out-of-order delivery".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "librtp/source/rtp-queue.c".into(),
                        start_line: 292,
                        end_line: 322,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "librtp/source/rtp-packet.c".into(),
                        start_line: 99,
                        end_line: 138,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q18: MP4 composition offsets
        RelevanceJudgment {
            id: "G2Q18".into(),
            query_text: "apply MP4 composition time offsets to presentation timestamps".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "libmov/source/mov-stts.c".into(),
                        start_line: 220,
                        end_line: 243,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "libmov/source/mov-stts.c".into(),
                        start_line: 204,
                        end_line: 218,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q19: FLV previous-tag-size codec
        RelevanceJudgment {
            id: "G2Q19".into(),
            query_text: "read and write FLV previous tag size fields".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "libflv/source/flv-header.c".into(),
                        start_line: 460,
                        end_line: 466,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "libflv/source/flv-header.c".into(),
                        start_line: 468,
                        end_line: 474,
                    },
                    RL::Related,
                ),
            ],
        },
        // G2Q20: FLV mux header output
        RelevanceJudgment {
            id: "G2Q20".into(),
            query_text: "encode FLV file header and tag headers for muxed output".into(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                (
                    SourceRange {
                        file: "libflv/source/flv-header.c".into(),
                        start_line: 273,
                        end_line: 288,
                    },
                    RL::Strong,
                ),
                (
                    SourceRange {
                        file: "libflv/source/flv-header.c".into(),
                        start_line: 290,
                        end_line: 319,
                    },
                    RL::Related,
                ),
            ],
        },
        // ==================== FZ: Fuzzy Queries (15 total) ====================
        // Lexical perturbations of G1 sources; same relevant_ranges.

        // FZ-G1Q1: flv_header_read
        RelevanceJudgment {
            id: "FZ-G1Q1-naming_case".into(),
            query_text: "flvHeaderRead in flv-header".into(),
            query_type: QueryType::Fuzzy,
            fuzzy_subtype: Some("naming_case".into()),
            relevant_ranges: vec![(
                SourceRange {
                    file: "libflv/source/flv-header.c".into(),
                    start_line: 25,
                    end_line: 44,
                },
                RL::Strong,
            )],
        },
        RelevanceJudgment {
            id: "FZ-G1Q1-abbrev_expand".into(),
            query_text: "flv header read in flv header".into(),
            query_type: QueryType::Fuzzy,
            fuzzy_subtype: Some("abbrev_expand".into()),
            relevant_ranges: vec![(
                SourceRange {
                    file: "libflv/source/flv-header.c".into(),
                    start_line: 25,
                    end_line: 44,
                },
                RL::Strong,
            )],
        },
        // FZ-G1Q2: mov_read_stts
        RelevanceJudgment {
            id: "FZ-G1Q2-synonym".into(),
            query_text: "parse method on mov-stts".into(),
            query_type: QueryType::Fuzzy,
            fuzzy_subtype: Some("synonym".into()),
            relevant_ranges: vec![(
                SourceRange {
                    file: "libmov/source/mov-stts.c".into(),
                    start_line: 7,
                    end_line: 33,
                },
                RL::Strong,
            )],
        },
        // FZ-G1Q3: rtp_packet_serialize
        RelevanceJudgment {
            id: "FZ-G1Q3-naming_case".into(),
            query_text: "rtpPacketSerialize in rtp-packet".into(),
            query_type: QueryType::Fuzzy,
            fuzzy_subtype: Some("naming_case".into()),
            relevant_ranges: vec![(
                SourceRange {
                    file: "librtp/source/rtp-packet.c".into(),
                    start_line: 140,
                    end_line: 150,
                },
                RL::Strong,
            )],
        },
        RelevanceJudgment {
            id: "FZ-G1Q3-paraphrase".into(),
            query_text: "serialize an RTP packet with header and payload".into(),
            query_type: QueryType::Fuzzy,
            fuzzy_subtype: Some("paraphrase".into()),
            relevant_ranges: vec![(
                SourceRange {
                    file: "librtp/source/rtp-packet.c".into(),
                    start_line: 140,
                    end_line: 150,
                },
                RL::Strong,
            )],
        },
        // FZ-G1Q4: rtsp_header_session
        RelevanceJudgment {
            id: "FZ-G1Q4-abbrev_expand".into(),
            query_text: "rtsp header session in rtsp header".into(),
            query_type: QueryType::Fuzzy,
            fuzzy_subtype: Some("abbrev_expand".into()),
            relevant_ranges: vec![(
                SourceRange {
                    file: "librtsp/source/rtsp-header-session.c".into(),
                    start_line: 11,
                    end_line: 47,
                },
                RL::Strong,
            )],
        },
        // FZ-G1Q5: rtmp_chunk_basic_header_read
        RelevanceJudgment {
            id: "FZ-G1Q5-naming_case".into(),
            query_text: "rtmpChunkBasicHeaderRead in rtmp-chunk-header".into(),
            query_type: QueryType::Fuzzy,
            fuzzy_subtype: Some("naming_case".into()),
            relevant_ranges: vec![(
                SourceRange {
                    file: "librtmp/source/rtmp-chunk-header.c".into(),
                    start_line: 21,
                    end_line: 40,
                },
                RL::Strong,
            )],
        },
        RelevanceJudgment {
            id: "FZ-G1Q5-paraphrase".into(),
            query_text: "decode RTMP chunk basic header with format and channel id".into(),
            query_type: QueryType::Fuzzy,
            fuzzy_subtype: Some("paraphrase".into()),
            relevant_ranges: vec![(
                SourceRange {
                    file: "librtmp/source/rtmp-chunk-header.c".into(),
                    start_line: 21,
                    end_line: 40,
                },
                RL::Strong,
            )],
        },
        // FZ-G1Q6: hls_playlist_duration
        RelevanceJudgment {
            id: "FZ-G1Q6-naming_affix".into(),
            query_text: "get_hls_playlist_duration method in hls-playlist".into(),
            query_type: QueryType::Fuzzy,
            fuzzy_subtype: Some("naming_affix".into()),
            relevant_ranges: vec![(
                SourceRange {
                    file: "libhls/source/hls-playlist.c".into(),
                    start_line: 3,
                    end_line: 15,
                },
                RL::Strong,
            )],
        },
        // FZ-G1Q7: mpeg_ts_add_stream
        RelevanceJudgment {
            id: "FZ-G1Q7-naming_affix".into(),
            query_text: "mpeg_ts_add_stream_ex in mpeg-ts-enc".into(),
            query_type: QueryType::Fuzzy,
            fuzzy_subtype: Some("naming_affix".into()),
            relevant_ranges: vec![(
                SourceRange {
                    file: "libmpeg/source/mpeg-ts-enc.c".into(),
                    start_line: 556,
                    end_line: 571,
                },
                RL::Strong,
            )],
        },
        RelevanceJudgment {
            id: "FZ-G1Q7-paraphrase".into(),
            query_text: "add an elementary stream to the MPEG-TS multiplexer".into(),
            query_type: QueryType::Fuzzy,
            fuzzy_subtype: Some("paraphrase".into()),
            relevant_ranges: vec![(
                SourceRange {
                    file: "libmpeg/source/mpeg-ts-enc.c".into(),
                    start_line: 556,
                    end_line: 571,
                },
                RL::Strong,
            )],
        },
        // FZ-G1Q8: sip_message_isinvite
        RelevanceJudgment {
            id: "FZ-G1Q8-naming_affix".into(),
            query_text: "sip_message_isinvite_check in sip-message".into(),
            query_type: QueryType::Fuzzy,
            fuzzy_subtype: Some("naming_affix".into()),
            relevant_ranges: vec![(
                SourceRange {
                    file: "libsip/src/sip-message.c".into(),
                    start_line: 367,
                    end_line: 370,
                },
                RL::Strong,
            )],
        },
        // FZ-G1Q9: sdp_h264_load
        RelevanceJudgment {
            id: "FZ-G1Q9-synonym".into(),
            query_text: "decode method on sdp-h264".into(),
            query_type: QueryType::Fuzzy,
            fuzzy_subtype: Some("synonym".into()),
            relevant_ranges: vec![(
                SourceRange {
                    file: "librtsp/source/sdp/sdp-h264.c".into(),
                    start_line: 59,
                    end_line: 83,
                },
                RL::Strong,
            )],
        },
        RelevanceJudgment {
            id: "FZ-G1Q9-abbrev_expand".into(),
            query_text: "sdp h264 load in sdp h264".into(),
            query_type: QueryType::Fuzzy,
            fuzzy_subtype: Some("abbrev_expand".into()),
            relevant_ranges: vec![(
                SourceRange {
                    file: "librtsp/source/sdp/sdp-h264.c".into(),
                    start_line: 59,
                    end_line: 83,
                },
                RL::Strong,
            )],
        },
        // FZ-G1Q10: pat_find
        RelevanceJudgment {
            id: "FZ-G1Q10-synonym".into(),
            query_text: "lookup method on mpeg-pat".into(),
            query_type: QueryType::Fuzzy,
            fuzzy_subtype: Some("synonym".into()),
            relevant_ranges: vec![(
                SourceRange {
                    file: "libmpeg/source/mpeg-pat.c".into(),
                    start_line: 177,
                    end_line: 186,
                },
                RL::Strong,
            )],
        },
    ]
}
