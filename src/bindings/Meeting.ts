// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Chapter } from "./Chapter";

export interface Meeting { uuid: string, title: string, company_name: string, company_id: string, prompt: string, summary: string, note: string, transcript: string, datetime: string, audio_path: string, published: boolean, publish_with_note: boolean | null, chapters: Array<Chapter>, }