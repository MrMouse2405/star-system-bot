<script lang="ts">
    import { onMount } from "svelte";
    import * as Select from "$lib/components/ui/select/index.js";
    import { setMode, resetMode } from "mode-watcher";

    // default to system to prevent hydration mismatch or flash
    let theme = $state<"light" | "dark" | "system">("system");

    onMount(() => {
        // specific to mode-watcher: checks if a user preference is saved
        const savedTheme = localStorage.getItem("mode-watcher-mode");

        if (savedTheme === "light" || savedTheme === "dark") {
            theme = savedTheme;
        } else {
            theme = "system";
        }
    });

    function onThemeChange(value: string) {
        theme = value as "light" | "dark" | "system";
        if (value === "system") {
            resetMode();
        } else {
            setMode(value as "light" | "dark");
        }
    }
</script>

<Select.Root type="single" value={theme} onValueChange={onThemeChange}>
    <Select.Trigger class="w-[180px]">
        {theme.charAt(0).toUpperCase() + theme.slice(1)}
    </Select.Trigger>
    <Select.Content>
        <Select.Item value="light">Light</Select.Item>
        <Select.Item value="dark">Dark</Select.Item>
        <Select.Item value="system">System</Select.Item>
    </Select.Content>
</Select.Root>
