<script lang="ts">
    import ThemeToggle from "./theme-toggle.svelte";
    import * as Card from "$lib/components/ui/card/index.js";
</script>

<div class="flex w-full max-w-md flex-col p-6">
    <Card.Root>
        <Card.Header class="flex flex-row items-center gap-4 space-y-0">
            <div class="flex flex-col gap-1">
                <Card.Title class="text-base">Theme</Card.Title>
                <Card.Description>
                    Select your preferred color mode.
                </Card.Description>
            </div>
            <ThemeToggle />
        </Card.Header>
    </Card.Root>
</div>
