import { useTranslation } from "react-i18next";

export const LanguageSwitcher = () => {
  const { i18n } = useTranslation();

  const toggleLanguage = () => {
    const currentLang = i18n.language;
    let newLang;
    
    switch (currentLang) {
      case "en":
        newLang = "ar";
        break;
      case "ar":
        newLang = "en";
        break;
      default:
        newLang = "en";
    }
    
    i18n.changeLanguage(newLang);
    localStorage.setItem("language", newLang);
  };

  const getLanguageLabel = () => {
    switch (i18n.language) {
      case "ar":
        return "EN";
      default:
        return "العربية";
    }
  };

  return (
    <button
      className="px-3 py-1 rounded-md border-2 border-[#212121] bg-transparent text-white hover:bg-[#212121] transition-colors"
      onClick={toggleLanguage}
    >
      {getLanguageLabel()}
    </button>
  );
};

export default LanguageSwitcher;
