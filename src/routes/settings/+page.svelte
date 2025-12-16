<script lang="ts">
    import ThemeToggle from "./theme-toggle.svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import {
        Select,
        SelectContent,
        SelectItem,
        SelectTrigger,
    } from "$lib/components/ui/select";
    import { Separator } from "$lib/components/ui/separator";
    // Import the full Form namespace to use Field, Item, Label, etc.
    import * as Form from "$lib/components/ui/form";
</script>

<h1 class="scroll-m-20 text-balance text-4xl font-extrabold tracking-tight">
    Settings
</h1>

<p class="text-muted-foreground text-xl leading-7 not-first:mt-6">
    Manage your bot configurations and preferences.
</p>
