import {
  Button,
  Modal,
  ModalContent,
  ModalHeader,
  ModalBody,
  ModalFooter,
  Select,
  SelectItem,
  useDisclosure,
} from "@nextui-org/react";
import { TbSettings } from "react-icons/tb";
import { useTranslation } from "react-i18next";

export type Theme = "system" | "light" | "dark";
export type Lang = "en" | "es" | "cn" | "de";

// TODO: docstring, once properties are finalized.
interface SettingsProps {
  defaultTheme?: Theme;
  onThemeChange?: (theme: React.ChangeEvent<HTMLSelectElement>) => void;
  defaultLang?: Lang;
  onLangChange?: (lang: React.ChangeEvent<HTMLSelectElement>) => void;
}

function Settings(props: SettingsProps) {
  const { t } = useTranslation();
  const { isOpen, onOpen, onOpenChange } = useDisclosure();
  const {
    defaultTheme,
    onThemeChange,
    defaultLang,
    onLangChange: onLanguageChange,
  } = props;

  return (
    <>
      <Button
        isIconOnly
        disableRipple
        onPress={onOpen}
        className="absolute bottom-2 left-2 transition-none"
      >
        <TbSettings />
      </Button>

      {/* TODO: make settings persist between sessions */}
      <Modal
        isOpen={isOpen}
        onOpenChange={onOpenChange}
        placement="top"
        backdrop="blur"
      >
        <ModalContent>
          {(onClose) => (
            <>
              <ModalHeader className="flex flex-col gap-1">
                {t("settings.header")}
              </ModalHeader>

              <ModalBody>
                {/* FIXME: disallow unselecting options */}
                <Select
                  label={t("settings.themeSelectLabel")}
                  defaultSelectedKeys={[defaultTheme || "system"]}
                  autoFocus
                  className="w-full"
                  aria-label={t("settings.themeSelectAriaLabel")}
                  onChange={onThemeChange}
                >
                  <SelectItem key="system">
                    {t("settings.themeSystem")}
                  </SelectItem>
                  <SelectItem key="light">
                    {t("settings.themeLight")}
                  </SelectItem>
                  <SelectItem key="dark">{t("settings.themeDark")}</SelectItem>
                </Select>

                {/* FIXME: disallow unselecting options */}
                {/* TODO: default to system locale instead of English */}
                <Select
                  label={t("settings.languageSelectLabel")}
                  defaultSelectedKeys={[defaultLang || "en"]}
                  className="w-full"
                  aria-label={t("settings.languageSelectAriaLabel")}
                  onChange={onLanguageChange}
                >
                  {/* TODO: make this more dynamic */}
                  <SelectItem key="en" value="en">
                    {t("settings.languageEnglish")}
                  </SelectItem>
                  <SelectItem key="es" value="es">
                    {t("settings.languageSpanish")}
                  </SelectItem>
                  <SelectItem key="cn" value="cn">
                    {t("settings.languageChinese")}
                  </SelectItem>
                  <SelectItem key="de" value="de">
                    {t("settings.languageGerman")}
                  </SelectItem>
                </Select>
              </ModalBody>

              <ModalFooter>
                <Button color="danger" variant="light" onPress={onClose}>
                  {t("settings.closeButton")}
                </Button>

                <Button color="primary" variant="light" onPress={onClose}>
                  {t("settings.saveButton")}
                </Button>
              </ModalFooter>
            </>
          )}
        </ModalContent>
      </Modal>
    </>
  );
}

export default Settings;
