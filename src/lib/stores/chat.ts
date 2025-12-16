// src/lib/stores/chat.ts
import { writable } from 'svelte/store';

// Define the message type
export type Message = {
    id: number;
    content: string;
    sender: 'user' | 'bot';
    timestamp: Date;
};

// Initialize the chat history with a welcome message
export const chatHistory = writable<Message[]>([
    {
        id: 0,
        content: "This is not a LLM! It only translates! Paste your message!",
        sender: 'bot',
        timestamp: new Date(),
    },
]);

let messageIdCounter = 1;

/**
 * Adds a new message to the chat history store.
 * @param content The message text.
 * @param sender 'user' or 'bot'.
 */
export function addMessage(content: string, sender: 'user' | 'bot') {
    const newMessage: Message = {
        id: messageIdCounter++,
        content,
        sender,
        timestamp: new Date(),
    };

    chatHistory.update(history => [...history, newMessage]);
}