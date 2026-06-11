import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import {
  Dropdown,
  SettingContainer,
  SettingsGroup,
  Slider,
  Textarea,
  ToggleSwitch,
} from "@/components/ui";
import { Input } from "../../ui/Input";
import { ShortcutInput } from "../ShortcutInput";
import { useSettings } from "../../../hooks/useSettings";

const KOKORO_VOICES = [
  { value: "af_heart", label: "Heart (US female)" },
  { value: "af_bella", label: "Bella (US female)" },
  { value: "af_nicole", label: "Nicole (US female, soft)" },
  { value: "af_sky", label: "Sky (US female)" },
  { value: "am_adam", label: "Adam (US male)" },
  { value: "am_michael", label: "Michael (US male)" },
  { value: "bf_emma", label: "Emma (UK female)" },
  { value: "bm_george", label: "George (UK male)" },
];

export const AssistantSettings: React.FC = () => {
  const { t } = useTranslation();
  const { settings, refreshSettings, updatePostProcessApiKey } = useSettings();

  const providers = settings?.post_process_providers || [];
  const selectedProviderId = settings?.assistant_provider_id || "custom";
  const selectedProvider = providers.find((p) => p.id === selectedProviderId);

  const [model, setModel] = useState("");
  const [systemPrompt, setSystemPrompt] = useState("");
  const [ttsPrompt, setTtsPrompt] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [baseUrl, setBaseUrl] = useState("");

  useEffect(() => {
    setModel(settings?.assistant_models?.[selectedProviderId] ?? "");
    setApiKey(settings?.post_process_api_keys?.[selectedProviderId] ?? "");
    setBaseUrl(selectedProvider?.base_url ?? "");
  }, [settings, selectedProviderId, selectedProvider]);

  useEffect(() => {
    setSystemPrompt(settings?.assistant_system_prompt ?? "");
  }, [settings?.assistant_system_prompt]);

  useEffect(() => {
    setTtsPrompt(settings?.assistant_tts_prompt ?? "");
  }, [settings?.assistant_tts_prompt]);

  const providerOptions = providers
    .filter((p) => p.id !== "apple_intelligence")
    .map((p) => ({ value: p.id, label: p.label }));

  const handleProviderSelect = async (providerId: string) => {
    await commands.setAssistantProvider(providerId);
    await refreshSettings();
  };

  const handleModelBlur = async () => {
    await commands.changeAssistantModelSetting(selectedProviderId, model);
    await refreshSettings();
  };

  const handlePromptBlur = async () => {
    await commands.changeAssistantSystemPromptSetting(systemPrompt);
    await refreshSettings();
  };

  const handleApiKeyBlur = async () => {
    await updatePostProcessApiKey(selectedProviderId, apiKey);
  };

  const handleBaseUrlBlur = async () => {
    await commands.changePostProcessBaseUrlSetting(selectedProviderId, baseUrl);
    await refreshSettings();
  };

  const handleTtsPromptBlur = async () => {
    await commands.changeAssistantTtsPromptSetting(ttsPrompt);
    await refreshSettings();
  };

  const setAndRefresh = async (promise: Promise<unknown>) => {
    await promise;
    await refreshSettings();
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title={t("settings.assistant.shortcuts.title")}>
        <ShortcutInput shortcutId="assistant" grouped={true} />
        <ShortcutInput shortcutId="assistant_vision" grouped={true} />
        <ShortcutInput shortcutId="assistant_panel_toggle" grouped={true} />
      </SettingsGroup>

      <SettingsGroup title={t("settings.assistant.provider.title")}>
        <SettingContainer
          title={t("settings.assistant.provider.providerLabel")}
          description={t("settings.assistant.provider.providerDescription")}
          descriptionMode="tooltip"
          layout="horizontal"
          grouped={true}
        >
          <Dropdown
            options={providerOptions}
            selectedValue={selectedProviderId}
            onSelect={handleProviderSelect}
          />
        </SettingContainer>

        {selectedProvider?.allow_base_url_edit && (
          <SettingContainer
            title={t("settings.assistant.provider.baseUrlLabel")}
            description={t("settings.assistant.provider.baseUrlDescription")}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <Input
              type="text"
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              onBlur={handleBaseUrlBlur}
              placeholder="https://my-resource.openai.azure.com/openai/v1"
              className="min-w-[380px]"
            />
          </SettingContainer>
        )}

        <SettingContainer
          title={t("settings.assistant.provider.apiKeyLabel")}
          description={t("settings.assistant.provider.apiKeyDescription")}
          descriptionMode="tooltip"
          layout="horizontal"
          grouped={true}
        >
          <Input
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            onBlur={handleApiKeyBlur}
            placeholder={t("settings.assistant.provider.apiKeyPlaceholder")}
            className="min-w-[320px]"
          />
        </SettingContainer>

        <SettingContainer
          title={t("settings.assistant.provider.modelLabel")}
          description={t("settings.assistant.provider.modelDescription")}
          descriptionMode="tooltip"
          layout="horizontal"
          grouped={true}
        >
          <Input
            type="text"
            value={model}
            onChange={(e) => setModel(e.target.value)}
            onBlur={handleModelBlur}
            placeholder={t("settings.assistant.provider.modelPlaceholder")}
            className="min-w-[320px]"
          />
        </SettingContainer>
      </SettingsGroup>

      <SettingsGroup title={t("settings.assistant.vision.title")}>
        <ToggleSwitch
          checked={settings?.assistant_screenshot_enabled ?? true}
          onChange={(checked) =>
            setAndRefresh(commands.setAssistantScreenshotEnabled(checked))
          }
          label={t("settings.assistant.vision.enableLabel")}
          description={t("settings.assistant.vision.enableDescription")}
          grouped={true}
        />
      </SettingsGroup>

      <SettingsGroup title={t("settings.assistant.tts.title")}>
        <ToggleSwitch
          checked={settings?.assistant_tts_enabled ?? false}
          onChange={(checked) =>
            setAndRefresh(commands.setAssistantTtsEnabled(checked))
          }
          label={t("settings.assistant.tts.enableLabel")}
          description={t("settings.assistant.tts.enableDescription")}
          grouped={true}
        />
        <SettingContainer
          title={t("settings.assistant.tts.voiceLabel")}
          description={t("settings.assistant.tts.voiceDescription")}
          descriptionMode="tooltip"
          layout="horizontal"
          grouped={true}
        >
          <Dropdown
            options={KOKORO_VOICES}
            selectedValue={settings?.assistant_tts_voice ?? "af_heart"}
            onSelect={(voice) =>
              setAndRefresh(commands.setAssistantTtsVoice(voice))
            }
            disabled={!settings?.assistant_tts_enabled}
          />
        </SettingContainer>
        <SettingContainer
          title={t("settings.assistant.tts.promptLabel")}
          description={t("settings.assistant.tts.promptDescription")}
          descriptionMode="tooltip"
          layout="stacked"
          grouped={true}
        >
          <Textarea
            value={ttsPrompt}
            onChange={(e) => setTtsPrompt(e.target.value)}
            onBlur={handleTtsPromptBlur}
            className="w-full"
            rows={3}
            disabled={!settings?.assistant_tts_enabled}
          />
        </SettingContainer>
      </SettingsGroup>

      <SettingsGroup title={t("settings.assistant.appearance.title")}>
        <SettingContainer
          title={t("settings.assistant.appearance.accentLabel")}
          description={t("settings.assistant.appearance.accentDescription")}
          descriptionMode="tooltip"
          layout="horizontal"
          grouped={true}
        >
          <Dropdown
            options={[
              {
                value: "violet",
                label: t("settings.assistant.appearance.accents.violet"),
              },
              {
                value: "blue",
                label: t("settings.assistant.appearance.accents.blue"),
              },
              {
                value: "emerald",
                label: t("settings.assistant.appearance.accents.emerald"),
              },
              {
                value: "rose",
                label: t("settings.assistant.appearance.accents.rose"),
              },
              {
                value: "amber",
                label: t("settings.assistant.appearance.accents.amber"),
              },
              {
                value: "mono",
                label: t("settings.assistant.appearance.accents.mono"),
              },
            ]}
            selectedValue={settings?.assistant_accent ?? "violet"}
            onSelect={(accent) =>
              setAndRefresh(commands.setAssistantAccent(accent))
            }
          />
        </SettingContainer>
        <SettingContainer
          title={t("settings.assistant.appearance.fontSizeLabel")}
          description={t("settings.assistant.appearance.fontSizeDescription")}
          descriptionMode="tooltip"
          layout="horizontal"
          grouped={true}
        >
          <Dropdown
            options={[
              {
                value: "small",
                label: t("settings.assistant.appearance.fontSizes.small"),
              },
              {
                value: "medium",
                label: t("settings.assistant.appearance.fontSizes.medium"),
              },
              {
                value: "large",
                label: t("settings.assistant.appearance.fontSizes.large"),
              },
            ]}
            selectedValue={settings?.assistant_font_size ?? "medium"}
            onSelect={(size) =>
              setAndRefresh(commands.setAssistantFontSize(size))
            }
          />
        </SettingContainer>
        <Slider
          value={settings?.assistant_panel_opacity ?? 1}
          onChange={(value) =>
            setAndRefresh(commands.setAssistantPanelOpacity(value))
          }
          min={0.5}
          max={1}
          step={0.05}
          label={t("settings.assistant.appearance.opacityLabel")}
          description={t("settings.assistant.appearance.opacityDescription")}
          grouped={true}
          formatValue={(v) => `${Math.round(v * 100)}%`}
        />
      </SettingsGroup>

      <SettingsGroup title={t("settings.assistant.systemPrompt.title")}>
        <SettingContainer
          title={t("settings.assistant.systemPrompt.label")}
          description={t("settings.assistant.systemPrompt.description")}
          descriptionMode="tooltip"
          layout="stacked"
          grouped={true}
        >
          <Textarea
            value={systemPrompt}
            onChange={(e) => setSystemPrompt(e.target.value)}
            onBlur={handlePromptBlur}
            className="w-full"
            rows={5}
          />
        </SettingContainer>
      </SettingsGroup>
    </div>
  );
};
