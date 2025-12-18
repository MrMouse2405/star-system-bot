<script lang="ts">
    import { onMount } from "svelte";
    import * as Chat from "$lib/components/ui/chat";
    import * as Avatar from "$lib/components/ui/avatar";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Badge } from "$lib/components/ui/badge";
    import { Alert, AlertDescription } from "$lib/components/ui/alert";
    import SendIcon from "@lucide/svelte/icons/send";
    import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
    import LanguagesIcon from "@lucide/svelte/icons/languages";
    import { invoke } from "@tauri-apps/api/core";
    import { UseAutoScroll } from "$lib/hooks/use-auto-scroll.svelte";

    import SoldierCat from "$lib/assets/soldier_cat.png";
    import Rhiel from "$lib/assets/Rhiel.webp";

    const STORAGE_KEY = "chat-history";
    const DRAFT_KEY = "chat-draft"; // New key for input draft

    interface TranslationResponse {
        language: string;
        translation: string;
    }

    type Message = {
        id: string;
        content: string;
        variant: "sent" | "received";
        timestamp: string;
        language?: string;
        isError?: boolean;
    };

    const autoScroll = new UseAutoScroll();

    const formatTime = (date: Date) => {
        return date.toLocaleTimeString("en-US", {
            hour: "numeric",
            minute: "2-digit",
            hour12: true,
        });
    };

    let userMessage = $state("");
    let messages = $state<Message[]>([]);
    let isTyping = $state(0);
    let isLoaded = false;

    // --- 1. Helper function to process the backend request ---
    // We extract this so we can call it from 'onMount' (recovery) and 'handleSendMessage' (normal)
    async function processBotResponse(textToTranslate: string) {
        isTyping += 1;
        try {
            const result = (await invoke("translate", {
                text: textToTranslate,
            })) as TranslationResponse;

            messages.push({
                id: crypto.randomUUID(),
                content: String(result.translation),
                variant: "received",
                timestamp: formatTime(new Date()),
                language: result.language,
            });

            autoScroll.scrollToBottom();
        } catch (error) {
            console.error("Translation error:", error);
            messages.push({
                id: crypto.randomUUID(),
                content: "Translation error: " + String(error),
                variant: "received",
                timestamp: formatTime(new Date()),
                isError: true,
            });
            autoScroll.scrollToBottom();
        } finally {
            isTyping -= 1;
        }
    }

    onMount(() => {
        // A. Load Chat History
        const savedHistory = localStorage.getItem(STORAGE_KEY);
        if (savedHistory) {
            try {
                messages = JSON.parse(savedHistory);
            } catch (e) {
                console.error("Failed to parse chat history", e);
            }
        }

        // B. Load Draft Input
        const savedDraft = localStorage.getItem(DRAFT_KEY);
        if (savedDraft) {
            userMessage = savedDraft;
        }

        // C. RECOVERY LOGIC: Check if we were interrupted while typing
        const lastMsg = messages[messages.length - 1];
        // If the last message is from the USER, the bot owes us a reply.
        if (lastMsg && lastMsg.variant === "sent") {
            console.log("Recovering interrupted response...");
            processBotResponse(lastMsg.content);
        }

        setTimeout(() => autoScroll.scrollToBottom(), 0);
        isLoaded = true;
    });

    // Save history AND draft on change
    $effect(() => {
        if (isLoaded) {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(messages));
            localStorage.setItem(DRAFT_KEY, userMessage);
        }
    });

    async function handleSendMessage(e: Event) {
        e.preventDefault();
        const trimmedMessage = userMessage.trim();
        if (!trimmedMessage) return;

        // 1. Add User Message
        messages.push({
            id: crypto.randomUUID(),
            content: trimmedMessage,
            variant: "sent",
            timestamp: formatTime(new Date()),
        });

        userMessage = ""; // Clear input (effect will save empty string to draft)

        // 2. Trigger Bot Response
        await processBotResponse(trimmedMessage);
    }
</script>

<div class="flex flex-col h-full w-full">
    <header class="flex shrink-0 items-center gap-3 border-b bg-background p-4">
        <Avatar.Root>
            <img src={SoldierCat} alt="bot profile" />
        </Avatar.Root>
        <div class="flex flex-col">
            <span class="text-sm font-medium">Star System Bot</span>
            <span class="text-xs text-muted-foreground">
                This is not a LLM! It only translates!
            </span>
        </div>
    </header>

    <div
        class="flex-1 min-h-0 overflow-y-auto scrollbar-hide"
        bind:this={autoScroll.ref}
    >
        <Chat.List class="h-full">
            {#each messages as msg (msg.id)}
                {#if msg.isError}
                    <div class="px-4 py-2">
                        <Alert
                            variant="destructive"
                            class="border-destructive/50"
                        >
                            <AlertCircleIcon class="h-4 w-4" />
                            <AlertDescription class="ml-2">
                                {msg.content}
                            </AlertDescription>
                        </Alert>
                        <div
                            class="text-xs text-muted-foreground mt-1 text-end"
                        >
                            {msg.timestamp}
                        </div>
                    </div>
                {:else}
                    <Chat.Bubble variant={msg.variant}>
                        <Chat.BubbleAvatar>
                            {#if msg.variant === "sent"}
                                <img src={Rhiel} alt="user profile" />
                            {:else}
                                <img src={SoldierCat} alt="bot profile" />
                            {/if}
                        </Chat.BubbleAvatar>
                        <Chat.BubbleMessage class="flex flex-col gap-2">
                            {#if msg.language && msg.variant === "received"}
                                <div class="flex items-center gap-1.5">
                                    <LanguagesIcon
                                        class="h-3.5 w-3.5 text-muted-foreground"
                                    />
                                    <Badge
                                        variant="secondary"
                                        class="text-xs font-normal"
                                    >
                                        Translated from {msg.language}
                                    </Badge>
                                </div>
                            {/if}
                            <p>{msg.content}</p>
                            <div
                                class="w-full text-xs group-data-[variant='sent']/chat-bubble:text-end text-muted-foreground"
                            >
                                {msg.timestamp}
                            </div>
                        </Chat.BubbleMessage>
                    </Chat.Bubble>
                {/if}
            {/each}
            {#if isTyping != 0}
                <Chat.Bubble variant="received">
                    <Chat.BubbleAvatar>
                        <img src={SoldierCat} alt="bot profile" />
                    </Chat.BubbleAvatar>
                    <Chat.BubbleMessage typing />
                </Chat.Bubble>
            {/if}
        </Chat.List>
    </div>

    <footer class="shrink-0 border-t bg-background">
        <form onsubmit={handleSendMessage} class="flex items-center gap-2 p-4">
            <Input
                bind:value={userMessage}
                class="flex-1"
                placeholder="Type your message..."
                required
            />
            <Button
                type="submit"
                size="icon"
                class="shrink-0"
                disabled={!userMessage.trim()}
            >
                <SendIcon class="h-5 w-5" />
            </Button>
        </form>
    </footer>
</div>
