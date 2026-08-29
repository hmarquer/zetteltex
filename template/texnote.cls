\NeedsTeXFormat{LaTeX2e}

\ProvidesClass{texnote}
\LoadClass[12pt]{article}

\RequirePackage{../../template/ztxbase}

% Enlace a documents.tex para referencias externas entre notas.
\ifx\HCode\UnDeFiNeD
    \subimport{../}{documents.tex}
\fi

\newcommand{\transclude}[2][note]{\ExecuteMetaData[#2.tex]{#1}}

% En article no existe \chapter; se estiliza como título en negrita.
\newcommand{\chapter}[1]{\textbf{\LARGE#1}}

% Referencias entre notas
\makeatletter
\newcommand{\exhyperref}[3][note]{%
    \@ifundefined{r@#1}%
        {% Referencia externa
            \ztxmaybehyperlink{%
                \ifx\HCode\UnDeFiNeD
                    \hyperref[#2-#1]{#3}%
                \else
                    \ztxhtmlhref{#2}{#2-#1}{#3}%
                \fi
            }{#3}%
        }%
        {% Referencia local (mismo comportamiento)
            \ztxmaybehyperlink{\hyperref[#1]{#3}}{#3}%
        }%
}

\newcommand{\excref}[2][note]{%
    \@ifundefined{r@#1}%
        {% Referencia externa
            \ifthenelse{\equal{#1}{note}}
                {\ztxmaybehyperlink{%
                    \ifx\HCode\UnDeFiNeD
                        \hyperref[#2-#1]{\texttt{#2}}%
                    \else
                        \ztxhtmlhref{#2}{#2-#1}{\texttt{#2}}%
                    \fi
                }{\texttt{#2}}}%
                {\ztxmaybehyperlink{%
                    \ifx\HCode\UnDeFiNeD
                        \exhyperref[#1]{#2}{\texttt{#2/}\cref*{#2-#1}}%
                    \else
                        \ztxhtmlhref{#2}{#2-#1}{\texttt{#2/#1}}%
                    \fi
                }{%
                    \ifx\HCode\UnDeFiNeD
                        \texttt{#2/}\cref*{#2-#1}%
                    \else
                        \texttt{#2/#1}%
                    \fi
                }}%
        }%
        {% Referencia local
            \ztxmaybehyperlink{%
                \ifthenelse{\equal{#1}{note}}
                    {\cref{#1}}
                    {\cref{#1}}%
            }{\ifthenelse{\equal{#1}{note}}{\cref{#1}}{\cref{#1}}}%
        }%
}

\newcommand{\exref}[2][note]{%
    \@ifundefined{r@#1}%
        {% Referencia externa
            \ifthenelse{\equal{#1}{note}}
                {\ztxmaybehyperlink{%
                    \ifx\HCode\UnDeFiNeD
                        \hyperref[#2-#1]{\texttt{#2}}%
                    \else
                        \ztxhtmlhref{#2}{#2-#1}{\texttt{#2}}%
                    \fi
                }{\texttt{#2}}}%
                {\ztxmaybehyperlink{%
                    \ifx\HCode\UnDeFiNeD
                        \exhyperref[#1]{#2}{\texttt{#2/}\ref*{#2-#1}}%
                    \else
                        \ztxhtmlhref{#2}{#2-#1}{\texttt{#2/#1}}%
                    \fi
                }{%
                    \ifx\HCode\UnDeFiNeD
                        \texttt{#2/}\ref*{#2-#1}%
                    \else
                        \texttt{#2/#1}%
                    \fi
                }}%
        }%
        {% Referencia local
            \ztxmaybehyperlink{%
                \ifthenelse{\equal{#1}{note}}
                    {\ref{#1}}
                    {\ref{#1}}%
            }{\ifthenelse{\equal{#1}{note}}{\ref{#1}}{\ref{#1}}}%
        }%
}

% En HTML las referencias se degradan a texto plano.
\ifx\HCode\UnDeFiNeD
\else
\let\ztxorig@href\href
\renewcommand{\href}[2]{%
    \ztxmaybehyperlink{\ztxorig@href{#1}{#2}}{#2}%
}
\let\ztxorig@hyperref\hyperref
\renewcommand{\hyperref}[2][]{%
    \ztxmaybehyperlink{\ztxorig@hyperref[#1]{#2}}{#2}%
}
\fi
\makeatother

\ProcessOptions